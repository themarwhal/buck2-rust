//! Bounds are restrictions applied to some types after they've been converted into the
//! `ty` form from the HIR.

use rustc_middle::ty::{self, ToPredicate, Ty, TyCtxt};
use rustc_span::Span;

/// Collects together a list of type bounds. These lists of bounds occur in many places
/// in Rust's syntax:
///
/// ```text
/// trait Foo: Bar + Baz { }
///            ^^^^^^^^^ supertrait list bounding the `Self` type parameter
///
/// fn foo<T: Bar + Baz>() { }
///           ^^^^^^^^^ bounding the type parameter `T`
///
/// impl dyn Bar + Baz
///          ^^^^^^^^^ bounding the forgotten dynamic type
/// ```
///
/// Our representation is a bit mixed here -- in some cases, we
/// include the self type (e.g., `trait_bounds`) but in others we do not
#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct Bounds<'tcx> {
    /// A list of region bounds on the (implicit) self type. So if you
    /// had `T: 'a + 'b` this might would be a list `['a, 'b]` (but
    /// the `T` is not explicitly included).
    pub region_bounds: Vec<(ty::Binder<'tcx, ty::Region<'tcx>>, Span)>,

    /// A list of trait bounds. So if you had `T: Debug` this would be
    /// `T: Debug`. Note that the self-type is explicit here.
    pub trait_bounds: Vec<(ty::PolyTraitRef<'tcx>, Span, ty::BoundConstness)>,

    /// A list of projection equality bounds. So if you had `T:
    /// Iterator<Item = u32>` this would include `<T as
    /// Iterator>::Item => u32`. Note that the self-type is explicit
    /// here.
    pub projection_bounds: Vec<(ty::PolyProjectionPredicate<'tcx>, Span)>,

    /// `Some` if there is *no* `?Sized` predicate. The `span`
    /// is the location in the source of the `T` declaration which can
    /// be cited as the source of the `T: Sized` requirement.
    pub implicitly_sized: Option<Span>,
}

impl<'tcx> Bounds<'tcx> {
    /// Converts a bounds list into a flat set of predicates (like
    /// where-clauses). Because some of our bounds listings (e.g.,
    /// regions) don't include the self-type, you must supply the
    /// self-type here (the `param_ty` parameter).
    pub fn predicates<'out, 's>(
        &'s self,
        tcx: TyCtxt<'tcx>,
        param_ty: Ty<'tcx>,
        // the output must live shorter than the duration of the borrow of self and 'tcx.
    ) -> impl Iterator<Item = (ty::Predicate<'tcx>, Span)> + 'out
    where
        'tcx: 'out,
        's: 'out,
    {
        // If it could be sized, and is, add the `Sized` predicate.
        let sized_predicate = self.implicitly_sized.and_then(|span| {
            // FIXME: use tcx.at(span).mk_trait_ref(LangItem::Sized) here? This may make no-core code harder to write.
            let sized = tcx.lang_items().sized_trait()?;
            let trait_ref = ty::Binder::dummy(tcx.mk_trait_ref(sized, [param_ty]));
            Some((trait_ref.without_const().to_predicate(tcx), span))
        });

        let region_preds = self.region_bounds.iter().map(move |&(region_bound, span)| {
            let pred = region_bound
                .map_bound(|region_bound| ty::OutlivesPredicate(param_ty, region_bound))
                .to_predicate(tcx);
            (pred, span)
        });
        let trait_bounds =
            self.trait_bounds.iter().map(move |&(bound_trait_ref, span, constness)| {
                let predicate = bound_trait_ref.with_constness(constness).to_predicate(tcx);
                (predicate, span)
            });
        let projection_bounds = self
            .projection_bounds
            .iter()
            .map(move |&(projection, span)| (projection.to_predicate(tcx), span));

        sized_predicate.into_iter().chain(region_preds).chain(trait_bounds).chain(projection_bounds)
    }
}
