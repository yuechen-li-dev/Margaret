document:
  name: rust-primer
  format: toon
  version: 1

intent:
  summary:
    Define the narrow Rust subset allowed in this repository.
    Reduce ownership pretzeling, avoid compiler-fighting designs, and keep code explicit, local, and easy to refactor.
  audience:
    primary:
      LLM authors
    secondary:
      human contributors
  non_goals:
    - This is not a general Rust tutorial.
    - This is not a showcase of every advanced ownership, lifetime, trait, or async feature.
    - This is not permission to fight the compiler until the code type-checks by accident.
    - This is not a license to replace simple design with borrow-checker appeasement rituals.
    - This is not an excuse to introduce abstraction mainly because the compiler complained.

goal:
  headline:
    Use boring, explicit Rust that keeps ownership, mutation, lifetimes, and error flow local and obvious.
  success_criteria:
    - Code is easy to review locally.
    - Refactors stay localized.
    - Ownership is visible from function signatures and type definitions.
    - Borrow scopes stay short.
    - The compiler works with the design instead of against it.
    - Authors do not introduce abstraction or indirection just to silence borrow checker errors.
    - The codebase remains understandable without reconstructing fragile lifetime proofs in the reviewer's head.

core_principles:
  - Prefer the dull solution.
  - Prefer owned data over delicate borrowed relationships.
  - Prefer short borrows over long-lived borrow coupling.
  - Prefer local mutation over distributed shared mutation.
  - Prefer plain structs and enums over trait architecture theater.
  - Prefer simple loops over iterator or combinator soup when clarity improves.
  - Prefer cheap cloning over lifetime pretzeling when cloning meaningfully simplifies the design.
  - Prefer compiler-friendly structure over cleverness.
  - Do not use a Rust feature merely because it exists.
  - Do not make the codebase more abstract at the cost of making it less trustworthy.
  - Do not fight the borrow checker with complexity when a simpler ownership model would solve the real problem.

must_do:
  - Target stable, modern Rust.
  - Keep ownership explicit.
  - Keep borrow scopes short.
  - Write code whose failure modes are visible in types and tests.
  - Use Result<T, E> and Option<T> idiomatically.
  - Follow existing repository-local patterns before introducing a new one.
  - Keep mutation and side effects visible.
  - Keep performance-sensitive work localized.
  - Prefer obvious correctness over borrow-checker appeasement cleverness.
  - Make it easy for a reviewer to tell what is owned, borrowed, mutated, cloned, and returned.
  - Split logic into steps when doing so shortens borrows or makes state transitions clearer.
  - Prefer designs the compiler can validate locally over designs that require global mental bookkeeping.

recommended:
  - Prefer owned structs by default.
  - Prefer passing references briefly instead of storing references long-term.
  - Prefer String over &str in owned data structures.
  - Prefer Vec<T>, HashMap<K, V>, and other standard library collections when they fit naturally.
  - Prefer enums for variants and state machines.
  - Prefer plain functions and inherent impl methods over traits when no interface boundary is needed.
  - Prefer simple for loops when they are clearer than iterator chains.
  - Prefer cloning small or cheap data when that reduces lifetime complexity.
  - Prefer IDs, indices, or handles over complicated internal reference graphs.
  - Prefer concrete types first and generic abstraction second.
  - Prefer narrow modules with obvious responsibilities.
  - Prefer data reshaping that simplifies ownership instead of preserving an awkward structure for theoretical purity.

not_recommended:
  - Long-lived borrowed fields in ordinary structs.
  - Self-referential design attempts.
  - Deep lifetime plumbing across many layers.
  - Trait-heavy architecture without a concrete interface need.
  - Generic abstraction introduced before duplication is real.
  - Iterator chains that are harder to read than a loop.
  - Shared mutable state as a default design habit.
  - Premature async design.
  - Clone avoidance that makes APIs and control flow substantially worse.
  - Error type theater for small modules.
  - Fighting the compiler by increasing abstraction instead of simplifying ownership.
  - Overusing builder or wrapper patterns to work around simple data flow problems.
  - Designing APIs around preserving borrows when moving owned data would be simpler.
  - Packing too much logic into chained expressions that accidentally lengthen borrow scope.

restricted:
  - unsafe is restricted.
  - Rc<RefCell<T>> is restricted.
  - Arc<Mutex<T>> and Arc<RwLock<T>> are restricted.
  - Interior mutability is restricted outside narrow justified cases.
  - Explicit lifetime-heavy public APIs are restricted unless the benefit is clear.
  - Trait object and dynamic dispatch use is restricted unless there is a real boundary need.
  - Async is restricted unless the repository is already async-first in that area.
  - Macro-heavy abstraction is restricted.
  - Complex generic bounds and advanced trait machinery are restricted.
  - Self-referential patterns are banned.
  - Compiler-fighting pinning, boxing, and future gymnastics are restricted outside established async boundaries.
  - Unsafe cell patterns and custom aliasing tricks are restricted.
  - Global mutable state is restricted.
  - Clever ownership wrappers invented only to satisfy one local borrow-checker complaint are restricted.

default_patterns:
  ownership:
    headline:
      Own data by default.
    rules:
      - Prefer owned data in structs.
      - Borrow briefly across function boundaries.
      - Do not store references in structs unless the design truly requires it.
      - Prefer handles, IDs, or indices over reference graphs when relationships must persist.
      - If ownership is unclear, simplify the design first.
      - Prefer transforming data into an owned representation rather than preserving an awkward borrowed one forever.
    guidance:
      - A reviewer should be able to see who owns what from local code.
      - If the design only works when everyone remembers an invisible lifetime story, the design is too fragile.

  borrowing:
    headline:
      Borrow briefly and locally.
    rules:
      - Keep borrow scopes short.
      - Avoid overlapping mutable and immutable borrows across large spans of code.
      - Split computation into steps if it shortens borrow lifetimes.
      - Introduce local variables or clones when they simplify borrow structure.
      - Do not widen borrow scope accidentally through chained expressions.
      - End borrows before the next mutation if possible.
    guidance:
      - Borrowing is a local tool, not a whole-program architecture style.
      - The easiest borrow to reason about is the one that ends quickly.

  cloning:
    headline:
      Cloning is a tool, not a sin.
    rules:
      - Prefer cheap or local clones when they remove lifetime coupling.
      - Do not contort APIs just to avoid cloning small values.
      - Do not clone blindly in hot paths without evidence.
      - Clone to simplify design, not to mask confusion.
      - Prefer a small honest clone over a large dishonest abstraction.
    guidance:
      - LLM authors often overreact to clone avoidance and make the code worse.
      - If a clone buys clarity, locality, and compiler-friendly structure, it is often the correct trade.

  structs_enums_and_traits:
    headline:
      Start with structs and enums. Reach for traits later.
    rules:
      - Prefer plain structs for data and clear behavior.
      - Prefer enums for variants and state.
      - Use traits when a real interface boundary exists.
      - Do not traitify everything by default.
      - Do not build abstraction layers before the code proves they are needed.
      - Prefer inherent impl methods when no polymorphic boundary exists.
    guidance:
      - Traits are useful, but they are not the default shape of ordinary code.
      - A concrete design that works well is better than a generic design that fights the compiler and the reader.

  generics:
    headline:
      Keep generics boring.
    rules:
      - Use generics to remove obvious duplication.
      - Prefer concrete types first.
      - Keep trait bounds small and readable.
      - Do not build type-level machinery for ordinary logic.
      - If a concrete implementation is clearer, use it.
      - Do not introduce a generic API just because Rust makes it possible.
    guidance:
      - Generics should reduce duplication, not increase the conceptual weight of the code.
      - If a reviewer has to decode the bounds before understanding the function, the abstraction is probably too expensive.

  iteration_and_control_flow:
    headline:
      Loops are respectable.
    rules:
      - Use iterator chains when they are truly clearer.
      - Use for loops when they make state and control flow easier to inspect.
      - Break large chains into named steps when clarity improves.
      - Do not write combinator soup to look idiomatic.
      - Prefer explicit local variables when they shorten lifetimes and clarify mutation.
    guidance:
      - Iterator style is not morally superior to a clear loop.
      - Rust code should not become harder to debug just to look elegant.

  error_handling:
    headline:
      Use Result and Option directly and honestly.
    rules:
      - Use Result<T, E> for fallible operations.
      - Use Option<T> for simple absence.
      - Use ? freely when it improves clarity.
      - Keep error types boring unless complexity is justified.
      - Do not hide failure in logs, globals, or side channels.
      - Do not build elaborate error hierarchies for small local modules.
    guidance:
      - Rust already has a good error story.
      - Use it plainly instead of inventing a more theatrical one.

  mutation_and_state:
    headline:
      Keep mutation local and visible.
    rules:
      - Prefer local mutable variables over shared mutable structures.
      - Do not introduce interior mutability to avoid understanding ownership.
      - Keep state transitions explicit.
      - Avoid designs that require many aliases to mutable state.
      - Prefer returning new owned values over coordinating complex shared updates when practical.
    guidance:
      - Most Rust pain comes from trying to distribute mutation too widely.
      - Simpler ownership usually beats more mutation infrastructure.

  modules_and_dependencies:
    headline:
      Keep module boundaries simple.
    rules:
      - Keep modules focused.
      - Prefer std and established dependencies first.
      - Do not create utility dumping grounds.
      - Do not add crates for trivial problems already solved clearly.
      - Keep dependency choices boring unless a real need exists.
    guidance:
      - The standard library already solves many ordinary problems well.
      - Extra crates add conceptual cost even when they save a few lines.

  async:
    headline:
      Async is not the default.
    rules:
      - Use async only where the repository already needs it.
      - Do not introduce async to look modern.
      - Do not mix shared mutable state with async casually.
      - Keep async boundaries explicit and narrow.
      - Do not spread async through modules that do not benefit materially from it.
    guidance:
      - Async Rust can multiply complexity very quickly.
      - If the design problem can stay synchronous, keep it synchronous.

  unsafe:
    headline:
      Unsafe must be rare, isolated, and justified.
    rules:
      - Do not write unsafe unless the repository truly requires it.
      - Keep unsafe blocks as small as possible.
      - Document the invariant that makes the unsafe code valid.
      - Do not use unsafe to escape a design problem that should be solved safely.
      - Do not let unsafe reasoning leak into ordinary callers more than necessary.
    guidance:
      - Unsafe is a narrow tool for narrow cases.
      - It is not a shortcut around compiler friction that simpler safe code could avoid.

portability_rules:
  - Code should compile and behave consistently on the repository's supported stable Rust toolchains.
  - Do not depend on unstable features or nightly-only behavior.
  - Keep platform-specific behavior isolated behind narrow boundaries when necessary.
  - Do not rely on compiler quirks or accidental inference behavior for correctness.

author_operating_rules:
  - First inspect nearby code and mirror established local patterns.
  - Do not introduce a second pattern for a problem the repository already solved.
  - Do not widen the repository subset during implementation.
  - Do not infer that a more advanced Rust feature is preferred just because it is available.
  - When multiple solutions are valid, choose the one with the clearest ownership.
  - When multiple solutions are valid, choose the one with the shortest borrow scope.
  - When multiple solutions are valid, choose the one with the least shared mutable state.
  - If a small clone removes large lifetime complexity, prefer the clone.
  - If a construct makes review harder, refactoring riskier, or compiler errors more global, avoid it.
  - If unsure, choose the simpler and more explicit construct.
  - Do not solve a local borrow-checker complaint by turning the whole module into indirection soup.
  - Do not mistake compiles for good fit for this repository.
  - Do not introduce Rc<RefCell<T>>, Arc<Mutex<T>>, or elaborate traits as emotional support.
  - Do not preserve an awkward ownership shape just because it was the first shape generated.

decision_rules:
  - Prefer code that is obviously correct over code that is merely borrow-checker-compliant.
  - Prefer owned values over complicated lifetime choreography.
  - Prefer one boring pattern used consistently over several elegant patterns mixed together.
  - Prefer local data flow over globally coupled references.
  - Prefer loops and named variables over expression density when clarity improves.
  - Prefer code that the compiler can validate locally over code that requires fragile invariants spread across the module.
  - Prefer small explicit state transitions over hidden aliasing and mutation channels.
  - If the clever solution and the boring solution are both valid, choose the boring solution.

examples:
  ownership_examples:
    headline:
      Show owned struct design contrasted with borrowed-field pretzeling.

  borrowing_examples:
    headline:
      Show short local borrows contrasted with broad borrow scope and chained-expression coupling.

  cloning_examples:
    headline:
      Show acceptable simplifying clones contrasted with clone avoidance that worsens the design.

  result_and_option_examples:
    headline:
      Show direct Result and Option usage contrasted with overcomplicated error handling.

  trait_and_generic_examples:
    headline:
      Show concrete-first design contrasted with premature trait and generic abstraction.

  iterator_vs_loop_examples:
    headline:
      Show clear loops contrasted with iterator soup.

  interior_mutability_examples:
    headline:
      Show explicit ownership and local mutation contrasted with premature Rc<RefCell<T>> or Arc<Mutex<T>> escape hatches.

  async_examples:
    headline:
      Show narrow justified async contrasted with unnecessary async spread and shared-state pain.

review_checklist:
  ask_before_merging:
    - Is ownership obvious?
    - Are borrow scopes short?
    - Is mutation local and visible?
    - Does this follow existing repository patterns?
    - Did this change introduce abstraction mainly to appease the compiler?
    - Would a small clone simplify this design materially?
    - Would a loop be clearer than this iterator chain?
    - Does this trait or generic abstraction solve a real present problem?
    - Is interior mutability or async being used because it is truly needed, or because the simpler ownership model was not chosen?
    - Can the important safety and lifetime reasoning be understood locally?
  reject_if:
    - The change introduces lifetime pretzeling without clear benefit.
    - The change introduces shared mutable state as a convenience shortcut.
    - The change introduces unnecessary trait or generic abstraction.
    - The change uses Rc<RefCell<T>>, Arc<Mutex<T>>, or unsafe without strong justification.
    - The change makes testing, review, refactoring, or compiler diagnostics meaningfully worse.
    - The change preserves a compiler-hostile ownership shape instead of simplifying it.

closing:
  message:
    This repository uses a narrow Rust subset on purpose.
    The goal is not to prove how much Rust you know.
    The goal is to produce explicit, compiler-friendly, testable code that is easy to review and does not require fighting the borrow checker to survive.
