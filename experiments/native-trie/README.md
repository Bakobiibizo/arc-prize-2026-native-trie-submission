# Clean native-reference trie seed

Requested by the operator after the reference-machine hot-path audit. This is a separate implementation written from scratch, borrowing concepts but no PathMap/MORK implementation or earlier experiment source. Cargo dependencies are empty. The Rust standard library supplies memory ownership and the physical per-node child container; it supplies no construction/evaluation services.

## Physical structure

A trie owns its nodes through ordinary child edges keyed by arbitrary byte-sequence tokens. Prefix paths are sequences of those tokens, not serialized/reconstructed addresses during execution. A child container (`BTreeMap`) is the trie's actual adjacency structure, not an extracted table, rule index, node-ID registry, or mirror of trie-held knowledge. Runtime instruction-field traversal borrows three constant edge keys; it does not enumerate the child map or rebuild a path from the root.

A node now shares its immutable incoming token label with its actual child-map key (`Rc<[u8]>`). Read-only label inspection lets output rendering read stored tokens directly, without inventing a host token registry. It is not a new execution instruction; `Machine::cycle` does not read labels and its source is unchanged from the original clean seed.

An **Address** is a direct handle to a physical node. A **reference edge** stores a weak native reference to another node. The owning child edges keep nodes alive; reference edges can form computation cycles without ownership leaks. A dropped foreign root yields a detected dangling reference, not unsafe memory access. There is no owner arena or lookup table consulted by execution. The node has three physical content possibilities: unset, explicit terminal, or a reference. These are bootstrap metadata, not domain operation codes.

State and output are two actual independently owned trie roots. Reference edges can point across them directly. No root-name prefix dispatch or address decoding is needed.

## The sole copy-and-follow cycle

An instruction node has three child cells: source, target, next. Their contents are references to cells. The physical cycle follows these existing edges, copies source cell contents to target, and follows the post-write next cell to the next instruction. The instruction pointer is itself a trie cell.

A terminal instruction address halts; missing fields, unset references and dangling references are faults. Copying terminal contents creates a new terminal. Instructions' own field cells are ordinary writable trie structure. Tests construct an instruction by copying the three fields of another, then enter the instruction built that way.

The host implementation uses only transient native node handles and marker checks during a cycle. It does not decode address strings, allocate address vectors, assemble host instruction records, evaluate domain operators, construct a variable-binding table, or enumerate datasets. Default execution returns only Advanced/Halted; diagnostic path/string materialization is not part of it.

To preserve atomic failure, the cycle validates the prospective continuation before writing: if target and next-cell are the same physical node, the prospective source content determines that continuation. Otherwise the unchanged next cell does. This uses node identity and a few transient native references—not root snapshots or rollback tables. Both descriptors and source contents are read before the two writes. This is single-threaded physical memory behavior, not a concurrent transaction guarantee.

## Defined terms

- **Child adjacency:** the edges belonging to an actual trie node.
- **Native handle:** a reference directly to that node; it is not a value that must be translated through a separate node registry.
- **Weak reference:** a native reference that does not keep its target alive by itself. Ownership stays with the physical trie; following an expired reference fails explicitly.
- **Bootstrap convention:** the physical reference, terminal marker, and source/target/next format needed to start execution. The seed still relies on this host-supplied physical cycle; zero allocation does not prove unrestricted self-representation.

## First bounded evidence

The fresh tests cover exact trie token/path identity, instruction construction/execution, changing its own source, post-write continuation, explicit terminal copying across roots, instruction-pointer aliasing, atomic faults, dangling references, and release of cyclic reference graphs.

The supplied counter fixture gives each bit a zero-state address and a one-state address. The count resides in the output trie as bit-state references. Those states contain transformations that write the opposite state and either return or follow the next bit. The runtime never tests a bit value or computes carry arithmetic. Fixture layout vectors are for setup/inspection only and are never read by Machine::cycle.

A separate thread-scoped allocator probe requires zero allocation/reallocation requests during execution for full and short carries at widths 1, 8, 64, 512, 4096, 32768, plus 256 successive eight-bit pulses. Setup/provisioning and output inspection allocate and are outside that claim. Counter arithmetic appears only in test oracles.

The benchmark records five fresh instances for both carry cases at each width, including all repetitions. Load, execution and inspection are separate; the external entry/pulse and final halt lookup are included in execution. Two-cycle measurements are timer/cache sensitive. No independent cold-process, cache-flush, CPU-pinning, disk, or cross-version causal-speedup claim is made. This new implementation does not need native CPU compiler flags.

## Nested construction programs

`constructions.rs` supplies seed programs, not runtime host evaluation. A **frame** is trie-held workspace for one unfinished application: parent address, current phase, operand entry references, and binding references. A **continuation** is an address specifying the next work, including resuming that parent. A **binding** here is a pair of native references to a computed fragment's first node and open endpoint; it is positional binding, not a general logical-variable unifier.

Shared trie programs perform the following: save a parent and select the left operand; bind the returned left result and select the right operand; connect the left endpoint to the returned right head, keep its endpoint, restore the parent and continue. A small record-access program reloads references to each active frame's slots. Indirect reads/writes are constructed from ordinary copy instructions that patch another instruction's own fields. No bind, join, mark, append, allocate or evaluate opcode was added.

For supplied C0/C1 applications, C0 joins two fragments; C1 uses the same program with a supplied literal `!`. The `mark (join bob alice)` fixture returns `bobalice!`; its intermediate `(first,end)` view still reads `bobalice` after append. Input is a supplied construction-application tree, not parsing arbitrary prefix-language text or discovering rules by their names.

Output locations and token fields are **reserved during setup**, with contents unset. Literal programs populate those fields with references to input-token nodes, successor references, and terminal markers. Joining changes the left terminal endpoint to a redirect to the next fragment. The final endpoint stays terminal. Rendering walks the already-computed output chain and reads its stored token labels; it does not concatenate operand values or evaluate an expression. Tokens are shared data nodes under the actual input trie, not an external alphabet table. Only tokens present in the input are provisioned.

The host seed assembler and setup vectors are discarded by `Builder::finish` before execution. `Evaluation::execute` is a bounded clock plus final-terminal validation; it does not inspect expression kinds or choose operands. Tests replace an operand reference after setup without rebuilding code, cover UTF-8, every byte including zero, empty fragments, left/right nesting, saved parent/binding references, missing operands, explicit early terminals and resource-limit outcomes. A final terminal other than the declared root return is reported as incomplete outer evaluation, not silently accepted as a correct result.

The expected work for these seed programs is `43 J + 10 L + 2 B` copy cycles, for J binary joins, L literal occurrences and B literal bytes. Tests/benchmarks require zero allocation requests during execution; setup and inspection are separate. Logical nesting is represented by trie-held parent references, not a host recursion stack. A fixed 64 KB host-stack test covers depth 512; the main depth grid goes through 4096.

## Compact Unicode character-prefix transfer

`prefixes.rs` selects actual stored constructions by the most-specific recorded prefix, preserving ambiguity and faults rather than enumerating rules. `character_prefixes.rs` applies the recovered scalar-identity construction: ten offline source-hashed prefix facts, one shared native value-to-value instruction. No scalar integers/range tests or Unicode service exist in runtime selection. Input is a supplied 21-bit binary path. Canonical underlying input values are interned in actual trie adjacency as opaque bit tokens; repeated values preserve native identity. This storage can allocate and is not a million-row output mapping or per-scalar program.

The exhaustive source-pinned run checks all 2^21 paths, executes the selected one-copy construction for all 1,112,064 valid scalars and rejects construction selection for the other 985,088 values. This proves primitive same-value scalar/character construction coverage, not full Unicode semantics or byte decoding. The opaque value identity is not a rendered glyph. Input representation, construction prefix and character-output contexts are distinct; shared native identities can feed typed aliases and word edges. See repository-root ../../docs/kernel/CHARACTER-PREFIX-CONTRACT.md and ../../docs/native/NATIVE-CHARACTER-PREFIX-../../docs/kernel/RESULTS.md for boundaries, source evidence, allocations and timing.

## Canonical-character word paths

`words.rs` owns a separate word trie whose physical edge keys are canonical native character identities, disjoint from metadata. No labels are read to assemble/match strings. Prefix nodes retain character/previous references; a completed endpoint is itself the canonical word identity. Completion records do not halt traversal, so `car` extends to `cars` while remaining complete. UTF-8/scalar aliases of equal characters share the same word nodes.

Supplied nonempty sequences carry checked character-origin references and a supplied boundary. Boundary plus ordered origin identities addresses the provenance episode; replay is idempotent, while distinct occurrences/derivations retain evidence. The word path carries the characters, and the episode path carries their origins. Bounded allocating inspection walks these already-stored paths together; no redundant constituent list or external string assembly is needed. Owners of lower character/source records must remain alive.

This is exact physical path recording, not automatic segmentation, an active-boundary authority check or the old lexical proposal/activation policy. The small fixture is not Unicode coverage. See repository-root ../../docs/kernel/WORD-TRIE-CONTRACT.md and ../../docs/native/NATIVE-WORD-../../docs/kernel/RESULTS.md for bounds, recovered reference source, measured evidence-layout change and the next Unicode-prefix transfer.

## Representation-selected character aliases

`representations.rs` places typed paths (`bits`, `utf8`, `scalar`, other supplied encodings), canonical character nodes, source observations and a shared value-to-value construction in the same actual state trie. It uses the generic record layout, not host decoding. Twelve explicitly shown associations in `support/alias_fixture.rs` teach five identities; this fixture is not a runtime decoder/table or full Unicode inventory. Lookup follows the actual taught paths. Given a native prefix, following the next edge needs no replay; completion is a recorded association and can coexist with longer completed paths.

Evidence retains representation-selector, path, original source-record and shared construction references. Character structure/semantics are separate nodes referenced from canonical identities. The optional common construction emits an already-resolved address with one native copy; lookup uses zero VM copies. Unknown paths are not automatically invalid encodings; no normalization or semantics is inferred. See repository-root ../../docs/native/NATIVE-REPRESENTATIONS-../../docs/kernel/RESULTS.md for bounds, raw benchmark provenance and next word-trie work.

## Reference recording and evidence

`records.rs` stores generic references at a supplied anchor: `holds/kind/candidates/target-identity`. The canonical candidate points to the target, links back to its anchor/slot, and owns an evidence index whose entries reference original caller-owned records. Same-target references converge; distinct evidence addresses accumulate; exact repeated evidence identity is idempotent. Distinct target identities remain alternatives, not a claim of semantic inequivalence. Resolution is Unknown, Unique or Ambiguous and never scans alternatives/evidence to choose an answer. Diagnostic exports may allocate and enumerate separately.

`prepare` builds a fresh subtree privately. `publish` adopts the complete candidate/evidence subtree with a single actual child-adjacency insertion, preserving its existing incoming key. Interleaved preparations reconcile with the current stored canonical entry. Ordinary public `provision` exposes a placeholder immediately and failed the invisible-preparation contrast experiment; that failure is retained in `evidence/native-records-development/`. The private storage publication helper is not a new copy-cycle opcode. All backlinks remain weak; release and dangling-owner tests cover the boundary.

Identity tokens are process-local native pointer bytes used only as opaque keys in actual adjacency. References are still followed directly; no execution ID decoder or external registry exists. They are explicitly **not persistent IDs**. Caller roots must retain targets/evidence; a deliberately expired owner produces a dangling fault. This does not authenticate/freeze evidence or prove different programs semantically distinct. Physical storage/record preparation can allocate and is not self-represented inference or generic induction.

The new generic layout is separate from the retained `ByteProbe` single-view/legacy lesson protocol. Use `records::resolve` for the new candidate/evidence sets; the old byte prototype's deferred-lesson behavior has not been silently rewritten. Tests include all 256 supplied byte relations with repeated sources in the new layout, while retaining every old prototype regression. `record_store` measures insertion, added evidence, exact repetition, unique/ambiguous/unknown selection separately. `record_refs` is a small trace.

## Ordinary addressing versus execution (operator correction)

`Address::locate` follows borrowed token keys through the actual child adjacency, without allocating a decoded path, provisioning missing nodes or running a copy program. `ByteProbe::locate_view` resolves a complete caller-delimited bit address; `view_at` takes an already supplied native anchor and only follows its stored byte-view reference. Neither executes a construction. Unknown paths/views remain errors; the helpers do not compute a byte from bits or synthesize a scalar.

`start_view` optionally executes the found view's existing getter: one copy emits its native address into the result trie. Its continuation is bound to the declared return; this mode does not produce the old stream protocol's status/anchor receipt fields. It is not a Unicode operation or new physical opcode. The helper is deliberately explicit about the changed boundary: ordinary trie addressing is a physical backend operation, not a simulated program of bit-by-bit copy instructions.

`address_lookup` benchmarks path lookup, given-anchor lookup and path-plus-emission separately. Its inputs are **already delimited addresses**, not an unframed bitstream, so the results must not be advertised as an end-to-end parser speedup over `byte_stream`. Batch timing includes collecting opaque native outputs into a reserved vector; inspection is outside timing. Given-anchor fixtures resolve their paths during separately recorded setup, not secretly for free. No per-query bit records are made. Old native lesson admission is retained for model initialization and comparison; this change does not complete generic model learning, reconciliation or streaming/framing.

## Shared bit anchors and byte views

`byte_views.rs` is the first bottom-up model probe. It adds a third, actually owned trie for canonical bit-prefix addresses. The two one-bit addresses are shared input symbols. At eight bits, `holds/raw-byte-msb-first` refers to an admitted byte-view record; its content points back to that same bit address. The record also retains its supplied lesson/source and interpretation-program references. It is a raw byte view, not UTF-8 validity or scalar decoding.

The host provisions a complete fixed eight-bit skeleton (511 prefixes, 256 candidate view slots) but initializes every slot as unknown. One trie-held admission program walks supplied bits and installs a shown view's backlink before publishing its forward reference. One read program walks fresh streams, invokes the stored view and emits its native reference. Unknown, EOF, truncated and repeated-admission outcomes are produced by trie programs. A repeated lesson, whether identical or contradictory, is explicitly deferred for reconciliation; no incumbent or observation is silently overwritten. Full evidence merging/ambiguity handling is not implemented.

**Transport record** means one supplied bit occurrence with an executable three-field header: source refers to the canonical zero/one data node, target/next refer to the same delivery port, and the record's content links the next occurrence. This is a fixed self-delivering input schema, not a generic reader for arbitrary passive trie data. The loader encodes supplied bits/lesson values only; no query-specific decoder or selected byte path is compiled. Tests change an input reference after loading and obtain the changed result with unchanged decoder/model definitions. Model selection and byte meaning are not computed in host code during execution. Host arithmetic in teacher/oracle/benchmark fixture generation is outside that computation.

Work is exactly 71 copies for first admission, 67 for a known byte or deferred repeated admission, 66 for an unknown byte, and `10 + 7r` for an exhausted input after `r` bits (`r=0` is EOF, `1..7` is truncation). A known stream of `n` bytes plus EOF costs `67n+10` copies. The eight-bit width and MSB-first interpretation are supplied model facts/schema, not new physical instructions. Reserving all 256 cases is bounded byte knowledge, not a proposed exponential state enumeration strategy for arbitrary-width values.

An interruption sweep tests every admission budget 0..71. Forward-first publication failed at copy 68; the preserved experiment is in `evidence/native-bit-byte/`. Backlink-first publication repairs the representation without snapshots or a new opcode. Whole admission is still not a transaction: a pending candidate/backlink may remain after interruption, but no published view lacks its backlink. Exact-copy-budget completion is detected by observing the explicit return marker, without performing an extra copy.

The assembler is discarded before execution. Input debug vectors are not consulted by the clock and can be dropped. Native byte output rendering only reads the selected record's stored label; it does not derive a number from bits. Allocation claims apply only to measured execution; input headers, full model provisioning and source records allocate during setup, and the byte benchmark measures those costs too.

This probe does not establish generic induction/schema creation, compact input storage, durable per-query provenance receipts, model sealing/write protection, cross-instance model sharing, recursive re-entry or fresh-node growth. Read tests compare model fields/view pointers before/after; this is not enforcement against the low-level mutation API. Supplied source IDs/records are retained, not authenticated physical observations. Scalar and character interpretation remain separate constructions, with no automatic scalar view filled by host Unicode decoding.

Run `target/native-trie/release/examples/byte_views` for the bounded trace and `target/native-trie/release/byte_stream` for all repeated byte rows. `scripts/record_native_byte.py NEW-DIRECTORY` records exact source hashes, all checks, raw samples and the retained seed/nested grids without retries.

## Not established

This remains an experiment, not completion of the kernel plan. General scoped variables/pattern matching, frame re-entry, fresh-address growth, persistence, evidence consolidation and induction remain unfinished. The physical copy convention and initial assembler are still externally supplied, not a proven minimal or fully self-hosted bootstrap.

Nodes and destinations are provisioned during setup. The runtime copies references/markers, not arbitrary child subtries. It cannot currently create an unbounded set of fresh nodes or learn an instruction meaning. The physical source/target/next convention remains external. These are unfinished research requirements, not criteria waived to call this seed a completed kernel.

Application occurrences own single-use mutable frames/endpoints. The supplied tree builder does not construct shared/re-entered frames or cyclic input; the low-level mutation API can create them, and no general admission validator is implemented. Provisioned workspace scales with input occurrences/bytes; unbounded fresh workspace is still unresolved. Literal data is loaded as token nodes and seed transfer programs; this is not yet a generic passive-data stream reader.

The old stream route's input amplification remains preserved as a failed direction for ordinary addressing: 208,162,776 cumulative requested bytes for 32,768 input bytes, with a node plus three descriptor cells per bit. The operator corrected that boundary; the addressed query route above does not build these records. Transport-window work is held rather than mistaken for a requirement of direct lookup. Generic duplicate-reference/evidence storage is now implemented in the separate record layout above; see repository-root ../../docs/native/NATIVE-RECORDS-../../docs/kernel/RESULTS.md. Representation-selected alias wiring is now implemented as described above; the separate canonical-character word trie is implemented as described above; next transfer the compact Unicode foundation/construction prefixes, not host semantic operations. The broader shared/re-entered application-frame probe remains held, not resolved by sequential byte-controller reuse. Preserve failures and try trie-program repairs before new primitives.

## Run from the research repository root

```sh
cargo test --manifest-path experiments/native-trie/Cargo.toml --target-dir target/native-trie
cargo build --release --manifest-path experiments/native-trie/Cargo.toml --target-dir target/native-trie
target/native-trie/release/native-trie-seed
cargo build --release --manifest-path experiments/native-trie/Cargo.toml --target-dir target/native-trie --bins --examples
target/native-trie/release/examples/nested
target/native-trie/release/nesting
```

Rust 1.95.0 is inherited from the repository's pinned toolchain. No code in the retained fallbacks is changed or linked by this crate.
