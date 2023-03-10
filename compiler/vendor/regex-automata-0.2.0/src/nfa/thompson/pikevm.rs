use alloc::{sync::Arc, vec, vec::Vec};

use crate::{
    nfa::thompson::{self, Error, State, NFA},
    util::{
        id::{PatternID, StateID},
        matchtypes::MultiMatch,
        sparse_set::SparseSet,
    },
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Config {
    anchored: Option<bool>,
    utf8: Option<bool>,
}

impl Config {
    /// Return a new default PikeVM configuration.
    pub fn new() -> Config {
        Config::default()
    }

    pub fn anchored(mut self, yes: bool) -> Config {
        self.anchored = Some(yes);
        self
    }

    pub fn utf8(mut self, yes: bool) -> Config {
        self.utf8 = Some(yes);
        self
    }

    pub fn get_anchored(&self) -> bool {
        self.anchored.unwrap_or(false)
    }

    pub fn get_utf8(&self) -> bool {
        self.utf8.unwrap_or(true)
    }

    pub(crate) fn overwrite(self, o: Config) -> Config {
        Config {
            anchored: o.anchored.or(self.anchored),
            utf8: o.utf8.or(self.utf8),
        }
    }
}

/// A builder for a PikeVM.
#[derive(Clone, Debug)]
pub struct Builder {
    config: Config,
    thompson: thompson::Builder,
}

impl Builder {
    /// Create a new PikeVM builder with its default configuration.
    pub fn new() -> Builder {
        Builder {
            config: Config::default(),
            thompson: thompson::Builder::new(),
        }
    }

    pub fn build(&self, pattern: &str) -> Result<PikeVM, Error> {
        self.build_many(&[pattern])
    }

    pub fn build_many<P: AsRef<str>>(
        &self,
        patterns: &[P],
    ) -> Result<PikeVM, Error> {
        let nfa = self.thompson.build_many(patterns)?;
        self.build_from_nfa(Arc::new(nfa))
    }

    pub fn build_from_nfa(&self, nfa: Arc<NFA>) -> Result<PikeVM, Error> {
        // TODO: Check that this is correct.
        // if !cfg!(all(
        // feature = "dfa",
        // feature = "syntax",
        // feature = "unicode-perl"
        // )) {
        if !cfg!(feature = "syntax") {
            if nfa.has_word_boundary_unicode() {
                return Err(Error::unicode_word_unavailable());
            }
        }
        Ok(PikeVM { config: self.config, nfa })
    }

    pub fn configure(&mut self, config: Config) -> &mut Builder {
        self.config = self.config.overwrite(config);
        self
    }

    /// Set the syntax configuration for this builder using
    /// [`SyntaxConfig`](crate::SyntaxConfig).
    ///
    /// This permits setting things like case insensitivity, Unicode and multi
    /// line mode.
    ///
    /// These settings only apply when constructing a PikeVM directly from a
    /// pattern.
    pub fn syntax(
        &mut self,
        config: crate::util::syntax::SyntaxConfig,
    ) -> &mut Builder {
        self.thompson.syntax(config);
        self
    }

    /// Set the Thompson NFA configuration for this builder using
    /// [`nfa::thompson::Config`](crate::nfa::thompson::Config).
    ///
    /// This permits setting things like if additional time should be spent
    /// shrinking the size of the NFA.
    ///
    /// These settings only apply when constructing a PikeVM directly from a
    /// pattern.
    pub fn thompson(&mut self, config: thompson::Config) -> &mut Builder {
        self.thompson.configure(config);
        self
    }
}

#[derive(Clone, Debug)]
pub struct PikeVM {
    config: Config,
    nfa: Arc<NFA>,
}

impl PikeVM {
    pub fn new(pattern: &str) -> Result<PikeVM, Error> {
        PikeVM::builder().build(pattern)
    }

    pub fn new_many<P: AsRef<str>>(patterns: &[P]) -> Result<PikeVM, Error> {
        PikeVM::builder().build_many(patterns)
    }

    pub fn config() -> Config {
        Config::new()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn create_cache(&self) -> Cache {
        Cache::new(self.nfa())
    }

    pub fn create_captures(&self) -> Captures {
        Captures::new(self.nfa())
    }

    pub fn nfa(&self) -> &Arc<NFA> {
        &self.nfa
    }

    pub fn find_leftmost_iter<'r, 'c, 't>(
        &'r self,
        cache: &'c mut Cache,
        haystack: &'t [u8],
    ) -> FindLeftmostMatches<'r, 'c, 't> {
        FindLeftmostMatches::new(self, cache, haystack)
    }

    // BREADCRUMBS:
    //
    // 1) Don't forget about prefilters.
    //
    // 2) Consider the case of using a PikeVM with an NFA that has Capture
    // states, but where we don't want to track capturing groups (other than
    // group 0). This potentially saves a lot of copying around and what not. I
    // believe the current regex crate does this, for example. The interesting
    // bit here is how to handle the case of multiple patterns...
    //
    // 3) Permit the caller to specify a pattern ID to run an anchored-only
    // search on.
    //
    // 4) How to do overlapping? The way multi-regex support works in the regex
    // crate currently is to run the PikeVM until either we reach the end of
    // the haystack or when we know all regexes have matched. The latter case
    // is probably quite rare, so the common case is likely that we're always
    // searching the entire input. The question is: can we emulate that with
    // our typical 'overlapping' APIs on DFAs? I believe we can. If so, then
    // all we need to do is provide an overlapping API on the PikeVM that
    // roughly matches the ones we provide on DFAs. For those APIs, the only
    // thing they need over non-overlapping APIs is "caller state." For DFAs,
    // the caller state is simple: it contains the last state visited and the
    // last match reported. For the PikeVM (and NFAs in general), the "last
    // state" is actually a *set* of NFA states. So I think what happens here
    // is that we can just force the `Cache` to subsume this role. We'll still
    // need some additional state to track the last match reported though.
    // Because when two or more patterns match at the same location, we need a
    // way to know to iterate over them. Although maybe it's not match index we
    // need, but the state index of the last NFA state processed in the cache.
    // Then we just pick up where we left off. There might be another match
    // state, in which case, we report it.

    pub fn find_leftmost_at(
        &self,
        cache: &mut Cache,
        haystack: &[u8],
        start: usize,
        end: usize,
        caps: &mut Captures,
    ) -> Option<MultiMatch> {
        let anchored =
            self.config.get_anchored() || self.nfa.is_always_start_anchored();
        let mut at = start;
        let mut matched_pid = None;
        cache.clear();
        'LOOP: loop {
            if cache.clist.set.is_empty() {
                if matched_pid.is_some() || (anchored && at > start) {
                    break 'LOOP;
                }
                // TODO: prefilter
            }
            if (!anchored && matched_pid.is_none())
                || cache.clist.set.is_empty()
            {
                self.epsilon_closure(
                    &mut cache.clist,
                    &mut caps.slots,
                    &mut cache.stack,
                    self.nfa.start_anchored(),
                    haystack,
                    at,
                );
            }
            for i in 0..cache.clist.set.len() {
                let sid = cache.clist.set.get(i);
                let pid = match self.step(
                    &mut cache.nlist,
                    &mut caps.slots,
                    cache.clist.caps(sid),
                    &mut cache.stack,
                    sid,
                    haystack,
                    at,
                ) {
                    None => continue,
                    Some(pid) => pid,
                };
                matched_pid = Some(pid);
                break;
            }
            if at >= end {
                break;
            }
            at += 1;
            cache.swap();
            cache.nlist.set.clear();
        }
        matched_pid.map(|pid| {
            let slots = self.nfa.pattern_slots(pid);
            let (start, end) = (slots.start, slots.start + 1);
            MultiMatch::new(
                pid,
                caps.slots[start].unwrap(),
                caps.slots[end].unwrap(),
            )
        })
    }

    #[inline(always)]
    fn step(
        &self,
        nlist: &mut Threads,
        slots: &mut [Slot],
        thread_caps: &mut [Slot],
        stack: &mut Vec<FollowEpsilon>,
        sid: StateID,
        haystack: &[u8],
        at: usize,
    ) -> Option<PatternID> {
        match *self.nfa.state(sid) {
            State::Fail
            | State::Look { .. }
            | State::Union { .. }
            | State::Capture { .. } => None,
            State::Range { ref range } => {
                if range.matches(haystack, at) {
                    self.epsilon_closure(
                        nlist,
                        thread_caps,
                        stack,
                        range.next,
                        haystack,
                        at + 1,
                    );
                }
                None
            }
            State::Sparse(ref sparse) => {
                if let Some(next) = sparse.matches(haystack, at) {
                    self.epsilon_closure(
                        nlist,
                        thread_caps,
                        stack,
                        next,
                        haystack,
                        at + 1,
                    );
                }
                None
            }
            State::Match { id } => {
                slots.copy_from_slice(thread_caps);
                Some(id)
            }
        }
    }

    #[inline(always)]
    fn epsilon_closure(
        &self,
        nlist: &mut Threads,
        thread_caps: &mut [Slot],
        stack: &mut Vec<FollowEpsilon>,
        sid: StateID,
        haystack: &[u8],
        at: usize,
    ) {
        stack.push(FollowEpsilon::StateID(sid));
        while let Some(frame) = stack.pop() {
            match frame {
                FollowEpsilon::StateID(sid) => {
                    self.epsilon_closure_step(
                        nlist,
                        thread_caps,
                        stack,
                        sid,
                        haystack,
                        at,
                    );
                }
                FollowEpsilon::Capture { slot, pos } => {
                    thread_caps[slot] = pos;
                }
            }
        }
    }

    #[inline(always)]
    fn epsilon_closure_step(
        &self,
        nlist: &mut Threads,
        thread_caps: &mut [Slot],
        stack: &mut Vec<FollowEpsilon>,
        mut sid: StateID,
        haystack: &[u8],
        at: usize,
    ) {
        loop {
            if !nlist.set.insert(sid) {
                return;
            }
            match *self.nfa.state(sid) {
                State::Fail
                | State::Range { .. }
                | State::Sparse { .. }
                | State::Match { .. } => {
                    let t = &mut nlist.caps(sid);
                    t.copy_from_slice(thread_caps);
                    return;
                }
                State::Look { look, next } => {
                    if !look.matches(haystack, at) {
                        return;
                    }
                    sid = next;
                }
                State::Union { ref alternates } => {
                    sid = match alternates.get(0) {
                        None => return,
                        Some(&sid) => sid,
                    };
                    stack.extend(
                        alternates[1..]
                            .iter()
                            .copied()
                            .rev()
                            .map(FollowEpsilon::StateID),
                    );
                }
                State::Capture { next, slot } => {
                    if slot < thread_caps.len() {
                        stack.push(FollowEpsilon::Capture {
                            slot,
                            pos: thread_caps[slot],
                        });
                        thread_caps[slot] = Some(at);
                    }
                    sid = next;
                }
            }
        }
    }
}

/// An iterator over all non-overlapping leftmost matches for a particular
/// infallible search.
///
/// The iterator yields a [`MultiMatch`] value until no more matches could be
/// found. If the underlying search returns an error, then this panics.
///
/// The lifetime variables are as follows:
///
/// * `'r` is the lifetime of the regular expression itself.
/// * `'c` is the lifetime of the mutable cache used during search.
/// * `'t` is the lifetime of the text being searched.
#[derive(Debug)]
pub struct FindLeftmostMatches<'r, 'c, 't> {
    vm: &'r PikeVM,
    cache: &'c mut Cache,
    // scanner: Option<prefilter::Scanner<'r>>,
    text: &'t [u8],
    last_end: usize,
    last_match: Option<usize>,
}

impl<'r, 'c, 't> FindLeftmostMatches<'r, 'c, 't> {
    fn new(
        vm: &'r PikeVM,
        cache: &'c mut Cache,
        text: &'t [u8],
    ) -> FindLeftmostMatches<'r, 'c, 't> {
        FindLeftmostMatches { vm, cache, text, last_end: 0, last_match: None }
    }
}

impl<'r, 'c, 't> Iterator for FindLeftmostMatches<'r, 'c, 't> {
    // type Item = Captures;
    type Item = MultiMatch;

    // fn next(&mut self) -> Option<Captures> {
    fn next(&mut self) -> Option<MultiMatch> {
        if self.last_end > self.text.len() {
            return None;
        }
        let mut caps = self.vm.create_captures();
        let m = self.vm.find_leftmost_at(
            self.cache,
            self.text,
            self.last_end,
            self.text.len(),
            &mut caps,
        )?;
        if m.is_empty() {
            // This is an empty match. To ensure we make progress, start
            // the next search at the smallest possible starting position
            // of the next match following this one.
            self.last_end = if self.vm.config.get_utf8() {
                crate::util::next_utf8(self.text, m.end())
            } else {
                m.end() + 1
            };
            // Don't accept empty matches immediately following a match.
            // Just move on to the next match.
            if Some(m.end()) == self.last_match {
                return self.next();
            }
        } else {
            self.last_end = m.end();
        }
        self.last_match = Some(m.end());
        Some(m)
    }
}

#[derive(Clone, Debug)]
pub struct Captures {
    slots: Vec<Slot>,
}

impl Captures {
    pub fn new(nfa: &NFA) -> Captures {
        Captures { slots: vec![None; nfa.capture_slot_len()] }
    }
}

#[derive(Clone, Debug)]
pub struct Cache {
    stack: Vec<FollowEpsilon>,
    clist: Threads,
    nlist: Threads,
}

type Slot = Option<usize>;

#[derive(Clone, Debug)]
struct Threads {
    set: SparseSet,
    caps: Vec<Slot>,
    slots_per_thread: usize,
}

#[derive(Clone, Debug)]
enum FollowEpsilon {
    StateID(StateID),
    Capture { slot: usize, pos: Slot },
}

impl Cache {
    pub fn new(nfa: &NFA) -> Cache {
        Cache {
            stack: vec![],
            clist: Threads::new(nfa),
            nlist: Threads::new(nfa),
        }
    }

    fn clear(&mut self) {
        self.stack.clear();
        self.clist.set.clear();
        self.nlist.set.clear();
    }

    fn swap(&mut self) {
        core::mem::swap(&mut self.clist, &mut self.nlist);
    }
}

impl Threads {
    fn new(nfa: &NFA) -> Threads {
        let mut threads = Threads {
            set: SparseSet::new(0),
            caps: vec![],
            slots_per_thread: 0,
        };
        threads.resize(nfa);
        threads
    }

    fn resize(&mut self, nfa: &NFA) {
        if nfa.states().len() == self.set.capacity() {
            return;
        }
        self.slots_per_thread = nfa.capture_slot_len();
        self.set.resize(nfa.states().len());
        self.caps.resize(self.slots_per_thread * nfa.states().len(), None);
    }

    fn caps(&mut self, sid: StateID) -> &mut [Slot] {
        let i = sid.as_usize() * self.slots_per_thread;
        &mut self.caps[i..i + self.slots_per_thread]
    }
}
