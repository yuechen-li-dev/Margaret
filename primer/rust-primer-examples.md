document:
  name: rust-primer-examples
  format: toon
  version: 1

intent:
  summary:
    Provide small example pairs that demonstrate the allowed Rust subset in practice.
    These examples are normative anchors for authors.
  audience:
    primary:
      LLM authors
    secondary:
      human contributors

examples:
  ownership:
    summary:
      Own data by default. Do not build ordinary designs out of long-lived borrowed relationships.

    good_owned_struct:
      why:
        The struct owns its data, so the type is easy to construct, move, and reason about.
      code: |
        #[derive(Debug, Clone)]
        struct UserRecord {
            user_id: String,
            display_name: String,
        }

        fn new_user(user_id: &str, display_name: &str) -> UserRecord {
            UserRecord {
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
            }
        }

    good_owned_collection:
      why:
        The container owns its contents and does not require lifetime plumbing across the module.
      code: |
        #[derive(Debug, Clone)]
        struct Project {
            name: String,
            files: Vec<String>,
        }

        impl Project {
            fn add_file(&mut self, path: String) {
                self.files.push(path);
            }
        }

    bad_borrowed_field_pretzel:
      why:
        The struct stores borrowed data unnecessarily, forcing lifetime coupling into ordinary code.
      code: |
        struct UserRecord<'a> {
            user_id: &'a str,
            display_name: &'a str,
        }

        fn new_user<'a>(user_id: &'a str, display_name: &'a str) -> UserRecord<'a> {
            UserRecord { user_id, display_name }
        }

    bad_reference_graph_shape:
      why:
        Persistent relationships are modeled with references instead of owned data or stable handles.
      code: |
        struct Node<'a> {
            name: &'a str,
            parent: Option<&'a Node<'a>>,
            children: Vec<&'a Node<'a>>,
        }

  borrowing:
    summary:
      Borrow briefly and locally. Do not let borrows sprawl across unrelated logic.

    good_short_borrow_scope:
      why:
        The immutable borrow ends before the next mutation, keeping the code easy for both compiler and reviewer.
      code: |
        fn append_summary(names: &mut Vec<String>) {
            let count = names.len();
            let summary = format!("count={count}");
            names.push(summary);
        }

    good_split_into_steps:
      why:
        Breaking work into steps shortens borrow lifetimes and makes state transitions easier to inspect.
      code: |
        fn uppercase_first(items: &mut [String]) {
            if items.is_empty() {
                return;
            }

            let upper = items[0].to_uppercase();
            items[0] = upper;
        }

    bad_chained_borrow_coupling:
      why:
        Chaining can accidentally widen borrow scope and make the code harder to modify safely.
      code: |
        fn append_summary(names: &mut Vec<String>) {
            let summary = names
                .first()
                .map(|name| format!("{name}-{}", names.len()))
                .unwrap_or_else(|| "empty".to_string());

            names.push(summary);
        }

    bad_borrow_spans_too_much_code:
      why:
        A reference is held across unrelated logic, increasing coupling and compiler friction.
      code: |
        fn update_and_log(values: &mut Vec<String>) {
            let first = &values[0];

            println!("first before update = {}", first);

            values.push("new".to_string());

            println!("first after update = {}", first);
        }

  cloning:
    summary:
      Cloning is allowed when it meaningfully simplifies ownership and lifetime structure.

    good_small_clone_for_clarity:
      why:
        A small clone removes lifetime coupling and keeps the code straightforward.
      code: |
        fn duplicate_first(items: &[String]) -> Option<(String, String)> {
            let first = items.first()?.clone();
            Some((first.clone(), first))
        }

    good_clone_before_mutation:
      why:
        Cloning a small value before mutation avoids awkward borrow overlap.
      code: |
        fn rename_first_with_suffix(items: &mut [String], suffix: &str) {
            if items.is_empty() {
                return;
            }

            let original = items[0].clone();
            items[0] = format!("{original}{suffix}");
        }

    bad_clone_avoidance_pretzel:
      why:
        The code preserves a borrow-heavy shape mainly to avoid a cheap clone.
      code: |
        fn rename_first_with_suffix<'a>(items: &'a mut [String], suffix: &'a str) -> &'a str {
            let first = &items[0];
            items[0] = format!("{first}{suffix}");
            &items[0]
        }

    bad_clone_everything_without_reason:
      why:
        Cloning can simplify design, but indiscriminate cloning creates unnecessary cost and noise.
      code: |
        fn total_lengths(items: &[String]) -> usize {
            items
                .iter()
                .cloned()
                .map(|item| item.clone())
                .map(|item| item.len())
                .sum()
        }

  result_and_option:
    summary:
      Use Result and Option directly and honestly. Keep error handling boring.

    good_option_for_absence:
      why:
        Option is the right fit when the only question is whether a value exists.
      code: |
        fn find_name(names: &[String], target: &str) -> Option<usize> {
            names.iter().position(|name| name == target)
        }

    good_result_with_simple_error:
      why:
        Result makes failure explicit without theatrical error architecture.
      code: |
        fn parse_positive_int(text: &str) -> Result<u32, String> {
            let value: u32 = text.parse().map_err(|_| "invalid integer".to_string())?;
            if value == 0 {
                return Err("value must be positive".to_string());
            }
            Ok(value)
        }

    good_question_mark_usage:
      why:
        The ? operator keeps fallible control flow direct and readable.
      code: |
        use std::fs;
        use std::path::Path;

        fn load_trimmed_text(path: &Path) -> Result<String, std::io::Error> {
            let text = fs::read_to_string(path)?;
            Ok(text.trim().to_string())
        }

    bad_overbuilt_error_theater:
      why:
        Small local logic does not need a dramatic custom error taxonomy.
      code: |
        enum ParsePositiveIntLayerOneError {
            Parse(ParsePositiveIntLayerTwoError),
        }

        enum ParsePositiveIntLayerTwoError {
            Validation(ParsePositiveIntLayerThreeError),
        }

        enum ParsePositiveIntLayerThreeError {
            Empty,
            Invalid,
            NonPositive,
        }

    bad_hidden_failure_side_channel:
      why:
        Logging is not a replacement for explicit failure in the type.
      code: |
        fn parse_positive_int(text: &str) -> u32 {
            match text.parse::<u32>() {
                Ok(value) if value > 0 => value,
                _ => {
                    eprintln!("failed to parse positive integer");
                    0
                }
            }
        }

  structs_enums_and_traits:
    summary:
      Start concrete. Use traits only when there is a real interface boundary.

    good_concrete_first_design:
      why:
        A plain struct with inherent methods is often the simplest correct shape.
      code: |
        struct Counter {
            value: usize,
        }

        impl Counter {
            fn new() -> Self {
                Self { value: 0 }
            }

            fn increment(&mut self) {
                self.value += 1;
            }

            fn value(&self) -> usize {
                self.value
            }
        }

    good_enum_for_state:
      why:
        Enums make variants explicit without requiring trait indirection.
      code: |
        enum JobState {
            Pending,
            Running,
            Finished,
            Failed(String),
        }

        fn is_terminal(state: &JobState) -> bool {
            matches!(state, JobState::Finished | JobState::Failed(_))
        }

    good_trait_for_real_boundary:
      why:
        A trait is justified when there is a genuine interface boundary.
      code: |
        trait FileStore {
            fn read_text(&self, path: &str) -> Result<String, String>;
        }

        struct LocalFileStore;

        impl FileStore for LocalFileStore {
            fn read_text(&self, path: &str) -> Result<String, String> {
                std::fs::read_to_string(path).map_err(|e| e.to_string())
            }
        }

    bad_traitify_everything:
      why:
        Trait indirection is introduced where a concrete type would be simpler and clearer.
      code: |
        trait CounterBehavior {
            fn increment(&mut self);
            fn value(&self) -> usize;
        }

        struct CounterImpl {
            value: usize,
        }

        impl CounterBehavior for CounterImpl {
            fn increment(&mut self) {
                self.value += 1;
            }

            fn value(&self) -> usize {
                self.value
            }
        }

    bad_premature_generic_abstraction:
      why:
        Generic machinery appears before the code proves it needs it.
      code: |
        struct Processor<TInput, TOutput, TError, THandler>
        where
            THandler: Fn(TInput) -> Result<TOutput, TError>,
        {
            handler: THandler,
        }

  iteration_and_control_flow:
    summary:
      Loops are respectable. Use iterator chains only when they are truly clearer.

    good_clear_loop:
      why:
        The loop makes control flow, mutation, and intermediate state obvious.
      code: |
        fn collect_even_strings(values: &[i32]) -> Vec<String> {
            let mut result = Vec::new();

            for value in values {
                if value % 2 == 0 {
                    result.push(value.to_string());
                }
            }

            result
        }

    good_small_iterator_chain:
      why:
        A short iterator chain is acceptable when it stays obvious.
      code: |
        fn collect_even_strings(values: &[i32]) -> Vec<String> {
            values
                .iter()
                .filter(|value| **value % 2 == 0)
                .map(|value| value.to_string())
                .collect()
        }

    bad_iterator_soup:
      why:
        The chain is harder to inspect and modify than a simple loop.
      code: |
        fn summarize(values: &[i32]) -> Vec<String> {
            values
                .iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    if *value > 0 && index % 2 == 0 {
                        Some((index, value.to_string()))
                    } else {
                        None
                    }
                })
                .map(|(index, text)| format!("{index}:{text}"))
                .collect()
        }

    bad_expression_density_borrow_confusion:
      why:
        Dense expressions can make borrow scope and state transitions harder to understand.
      code: |
        fn rewrite_first(items: &mut Vec<String>) {
            items[0] = items
                .first()
                .map(|value| value.trim().to_uppercase())
                .unwrap_or_else(|| "EMPTY".to_string());
        }

  interior_mutability:
    summary:
      Interior mutability is a narrow tool, not a default escape hatch.

    good_explicit_local_mutation:
      why:
        Mutation is local, visible, and does not require shared ownership machinery.
      code: |
        struct Queue {
            items: Vec<String>,
        }

        impl Queue {
            fn push(&mut self, item: String) {
                self.items.push(item);
            }

            fn pop(&mut self) -> Option<String> {
                self.items.pop()
            }
        }

    good_handle_based_design:
      why:
        Stable handles avoid persistent reference graphs and shared mutable borrowing problems.
      code: |
        struct Graph {
            names: Vec<String>,
        }

        impl Graph {
            fn add_node(&mut self, name: String) -> usize {
                self.names.push(name);
                self.names.len() - 1
            }

            fn name(&self, id: usize) -> Option<&str> {
                self.names.get(id).map(String::as_str)
            }
        }

    bad_rc_refcell_emotional_support:
      why:
        Shared mutable state is introduced mainly because ownership was not simplified first.
      code: |
        use std::cell::RefCell;
        use std::rc::Rc;

        struct Node {
            name: Rc<RefCell<String>>,
        }

        fn rename(node: &Node, new_name: &str) {
            *node.name.borrow_mut() = new_name.to_string();
        }

    bad_arc_mutex_by_default:
      why:
        Concurrency-oriented shared mutation appears before the design proves it is needed.
      code: |
        use std::sync::{Arc, Mutex};

        struct AppState {
            users: Arc<Mutex<Vec<String>>>,
        }

  async:
    summary:
      Async is not the default. Use it only where the repository truly needs it.

    good_sync_by_default:
      why:
        A synchronous design is simpler and should remain synchronous when it is good enough.
      code: |
        use std::fs;
        use std::path::Path;

        fn load_config(path: &Path) -> Result<String, std::io::Error> {
            fs::read_to_string(path)
        }

    good_narrow_async_boundary:
      why:
        Async is confined to a boundary that genuinely benefits from it.
      code: |
        async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
            let response = client.get(url).send().await?;
            response.text().await
        }

    bad_async_spread:
      why:
        Async spreads through code that does not materially benefit from it.
      code: |
        async fn parse_name(text: &str) -> String {
            text.trim().to_string()
        }

        async fn build_message(text: &str) -> String {
            let name = parse_name(text).await;
            format!("hello {name}")
        }

    bad_async_shared_state_pain:
      why:
        Async is mixed with shared mutable state in a way that multiplies complexity.
      code: |
        use std::sync::{Arc, Mutex};

        struct AppState {
            jobs: Arc<Mutex<Vec<String>>>,
        }

        async fn add_job(state: Arc<AppState>, name: String) {
            state.jobs.lock().unwrap().push(name);
        }

  unsafe:
    summary:
      Unsafe must be rare, isolated, and justified.

    good_safe_standard_code:
      why:
        The standard library already solves the problem safely and clearly.
      code: |
        fn first_byte(data: &[u8]) -> Option<u8> {
            data.first().copied()
        }

    bad_unsafe_for_no_reason:
      why:
        Unsafe is introduced without a real boundary need or documented invariant.
      code: |
        fn first_byte(data: &[u8]) -> u8 {
            unsafe { *data.get_unchecked(0) }
        }

review_hints:
  summary:
    Use these examples directionally, not mechanically.
  rules:
    - Prefer designs that own data cleanly over designs that preserve awkward borrowed shapes.
    - Prefer short borrows, split steps, and named locals when they simplify compiler reasoning.
    - Prefer small clones over large lifetime pretzels.
    - Prefer concrete structs and enums over premature trait and generic abstraction.
    - Prefer loops over iterator chains when clarity improves.
    - Treat Rc<RefCell<T>>, Arc<Mutex<T>>, async spread, and unsafe as escalation points, not defaults.

closing:
  message:
    These examples are anchors, not decoration.
    The goal is not impressive Rust.
    The goal is trustworthy Rust that the compiler can validate without turning the code into a pretzel factory.
