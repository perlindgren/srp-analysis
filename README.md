# Lab4, Suggested solution

## Optional for higher grades

- Use `clap` for command line parsing making your application work as a "professional" tool. This allows you to easily implement a `-h/--help` to document your tool and its use.

- Generate report on the analysis results, this could be as a generated html/xml or however you feel like results are best reported and visualized.

- Characterize scheduling overhead of RTIC for:

  - dispatching hardware tasks.

  - dispatching software tasks.

  Make a new repository based on the `d7020e_lab1` repository where you implement cycle accurate benchmarking for the above.

- Extend the task/resource model in the Lab4 analyzer to distinguish between software and hardware tasks, and take the OH into account for the analysis (including the OH scaling of message payload size for software tasks).

Hint. For simplicity you can measure the round trip time for hardware and software tasks (from the `idle` task).

The round trip time for hardware task will include the `pend` and the hardware dispatch mechanism. For software tasks the round trip time will include the `spawn` (internal queuing of the message payload, the `pend` of the dispatcher and the hardware dispatch mechanism), together with the dispatcher dequeuing the message payload and running the task.

The round trip time provides a safe over-approximation of the dispatch overhead.

Try to characterize the overhead in terms of payload size (you can try with an empty message, a 4 byte message and a 32 byte message, to derive a formula for the OH scaling.)

Since we are just interested in the scheduling overhead the task itself should be empty.

---

## Preparation

Start by reading 1, 2 and 3:

1. [A Stack-Based Resource Allocation Policy for Realtime Processes](https://www.math.unipd.it/~tullio/RTS/2009/Baker-1991.pdf), which refers to

2. [Stack-Based Scheduling of Realtime Processes](https://link.springer.com/content/pdf/10.1007/BF00365393.pdf), journal publication based on technical report [3] of the 1991 paper. The underlying model is the same in both papers.

3. [Rate Monotonic Analysis](http://www.di.unito.it/~bini/publications/2003BinButBut.pdf), especially equation 3 is of interest to us. (It should be familiar for the real-time systems course you have taken previously.)

---

## SRP Analysis for RTIC applications

---

### Definitions

A task `t` is defined by:

- `P(t)` the priority of task `t`
- `D(t)` the deadline of task `t`
- `A(t)` the (minimum) inter-arrival of task `t`

A resource `r` is defined by:

- `π(r)` the resource ceiling, computed as the highest priority of any task accessing the resource `r`. SRP allows for dynamic priorities (e.g. EDF), however in our case we have static priorities only.

For SRP based analysis we assume a task to perform/execute a finite sequence of operations/instructions (aka. run-to-end/run-to-completion semantics). During execution, a task can claim (lock) resources in a nested fashion. Sequential re-claim of resources is allowed but NOT re-claiming an already held (locked) resource (since that would violate the Rust memory aliasing rule - re-claiming the same resource twice gives you two mutable references to the same resource).

Let `...` denote a sequence of instructions, and `[r:...]` denote a sequence of instructions holding the resource `r` (i.e. a critical section for the resource `r`).

A possible trace for a task can look like:

`t1:...[r1:...[r2:...]...]...[r2:...]...`. In this case the task `t1` starts and at some point claims `r1` and inside the critical section claims `r2` (nested claim), at some point it exits `r2`, exits `r1` and continues executing and enters a critical section on `r2`, and then finally executes until completion. We see that `t1` holds `r1` once and `r2` twice during its execution (but never holds `r2` while already holding `r2`).

In `src/common.rs` a data structure for expressing traces is given, and in `src/main.rs` an example set of task traces is provided.

---

## Schedulability Analysis

---

### Worst Case Execution Time (WCET)

- `C(t)` the Worst Case Execution Time (WCET) for Task `t`

In general determining WCET is rather tricky, especially for systems with advanced memory hierarchies (caches) and multiple cores/CPUs. In our case we target single core architectures without advanced caching, hence a measurement based approach will give a very accurate estimate.

In this course, we have seen the use of symbolic execution to automatically generate concrete tests triggering each feasible execution path.

To correctly take concurrency into account resource state can be treated symbolically. Thus, for a critical section, the resource is given a fresh (new) symbolic value for each critical section. Inside the critical section we are ensured exclusive access (and thus the value can be further constrained inside of the critical section).

We can model hardware (peripheral register accesses) as shared resources (shared between your application and the environment). As such each _read_ regenerates a new symbolic value while write operations have no side-effect. Luckily, in the Rust embedded ecosystem, all register accesses are done through `volatile_register`, thus by overriding [volatile_register](https://docs.rs/volatile-register/latest/volatile_register/) we can get fresh (new) symbolic values for each register read (during analysis writing to a `volatile_register` is a no-operation).

For now, we just assume we have complete WCETs information in terms of `start` and `end` time-stamps (`u32`) for each section `_: ...`, see the `Trace` data structure in `common.rs` (a section can be the complete task as well as an inner critical section).

---

### Total CPU request (or total load factor)

Each task `t` has a WCET `C(t)` and given (assumed) inter-arrival time `A(t)`. The CPU request (or load) inferred by a task is `L(t) = C(t) / A(t)`. Ask yourself, what is the consequence of `C(t) > A(t)`?

We can compute the total CPU request (or load factor), as `Ltot = sum(L(T))`, `T` being the set of tasks `T: {t1..tn}`.

Ask yourself, what is the consequence of `Ltot > 1`?

Implement a function taking the set of tasks `T` (represented by `Vec<Task>`) and returning the load factor. (Use data structures from `common.rs` for suitable data structures and inspiration.)

---

### Response time (simple over-approximation)

Under SRP response time can be computed by equation 7.22 in [Hard Real-Time Computing Systems](https://doc.lagout.org/science/0_Computer%20Science/2_Algorithms/Hard%20Real-Time%20Computing%20Systems_%20Predictable%20Scheduling%20Algorithms%20and%20Applications%20%283rd%20ed.%29%20%5BButtazzo%202011-09-15%5D.pdf).

In general the response time for a task `t` is computed as.

- `R(t) = B(t) + C(t) + I(t)`, where
  - `B(t)` is the blocking time for task `t`, and
  - `I(t)` is the interference (preemptions) to task `t`

For a task set to be schedulable under SRP we have two requirements:

- `Ltot < 1`
- `R(t) < D(t)`, for all tasks `t`. (`R(t) >= D(t)` implies a deadline miss.)

---

#### Blocking

SRP brings the outstanding property of single blocking. In other words, a task `t` is blocked by the maximal critical section a task `l` with lower priority (`P(l) < P(t)`) holds a resource `l_r`, with a ceiling `π(l_r)` equal or higher than the priority of `t` (`π(l_r) >= P(t)`).

- `B(t) = max(C(l_r))`, where `P(l) < P(t)`, `π(l_r) >= P(t)`

Implement a function that takes a Task `t` together with the system wide set of tasks `T`, and returns the corresponding blocking time `B(t)`.

---

#### Busy period

- `Bp(t)` the busy period for Task `t` denotes the duration from task arrival (request for execution) until finish.

Intuitively, during the busy period `Bp(t)` each higher priority task `h` (`P(h) > P(t)`) may preempt `⌈ Bp(t) / A(h) ⌉` (ceiling, i.e. rounded upwards) number of times.

---

#### Preemptions, over approximation

We can over approximate the `Bp(t) = D(t)` (assuming the _worst permissible busy-period_).

- `I(t) = sum(C(h) * ceiling(Bp(t) / A(h)))`, forall tasks `h`, `P(h) > P(t)`

As a technical detail. For the scheduling of tasks of the same priority, the original work on SRP adopted a FIFO model (first arrived, first served). Under Rust RTIC, for efficiency tasks are bound directly to interrupts and scheduled by the underlying hardware. The hardware schedules tasks based on priority, on tie (same priority) tasks are scheduled based on priority group, and on tie of priority group based on vector index number. For sake of simplicity we assume all tasks to have unique priorities for this exercise, although detailed analysis is possible taking vector indexing into account (priority grouping is not used by RTIC). A better approximation is thus possible treating tasks at the same priority as interfering each other. (Technically tasks on the same priority level will not preempt each other, but from a scheduling viewpoint the worst case effect can be seen as interference.)

Implement a function that takes a Task `t` together with the system wide set of tasks `T` and returns the corresponding preemption time `I(t)`.

Now implement a function that computes the response time for a task `t` (for the system wide set of tasks `T`), as `R(t) = B(t) + C(t) + I(t)`.

Finally, make a function that iterates over the task set and returns a vector with containing:
`Vec<Task, R(t), C(t), B(t), I(t)>`. Just a simple `println!` of that vector gives the essential information on the analysis.

---

#### Preemptions revisited

The _busy-period_ is in `7.22` (Hard Real-Time Computing Systems) computed by a recurrence equation. Intuitively, the response time `R(t) = B(t) + C(t) + I(t, R(t))`, where `I(t,R(t))` denotes the interference (preemption time) during the _busy period_ (`R(t)`). We iteratively compute `R(t)`, starting from `R(t) = B(t) + C(t)` (the time without interference/preemptions) until a fix-point is reached.

Implement the recurrence relation (equation) starting from `R(t) = B(t) + C(t)`. The recurrence might diverge in case `Bp(t) > D(t)`, this is a pathological case, where the task becomes non-schedulable. In that case terminate the recurrence (with an error indicating a deadline miss). You might want to indicate that a non feasible response time have been reached by using the `Result<u32, ())>` type or some other means e.g., (`Option<u32>`).

You can let your `preemption` function take a parameter indicating if the exact solution or approximation should be used.

---

## Resources

`common.rs` gives the basic data structures, and some helper functions.

`main.rs` gives an example on how `Tasks` can be manually constructed. This is vastly helpful for your development, when getting started.

## Tips

When working with Rust, the standard library documentation [std](https://doc.rust-lang.org/std/) is excellent and easy to search (just press S). For most cases, you will find examples on intended use and cross referencing to related data types is just a click away.

Use the `main` example to get started. Initially you may simplify it further by reducing the number of tasks/and or resources. Make sure you understand the helper functions given in `common.rs`, (your code will likely look quite similar). You might want to add further `common` types and helper functions to streamline your development, along the way.

Generate your own task sets to make sure your code works in the general case not only for the `Tasks` provided.

Use the built in test framework to formulate unit tests for your code.

---

## Learning outcomes, Robust and Energy Efficient Real-Time Systems

- Real-Time Scheduling and Analysis. SRP provides an execution model and resource management policy with outstanding properties of race-and deadlock free execution, single blocking and stack sharing. Our Rust RTIC framework provides a correct by construction implementation of SRP, exploiting zero-cost (software) abstractions. Using Rust RTIC resource management and scheduling are performed directly by the hardware which allows for efficiency (Rust zero-cost abstraction) and predictability.

  The SRP model is amenable to static analysis, which you have now internalized through an actual implementation of the theoretical foundations. We have also covered methods for Worst Case Execution Time (WCET) analysis by cycle accurate measurements, which in combination with Symbolic Execution for test-case generation allows for high degree of automation.

- Energy Consumption is roughly proportional to the supply voltage (static leakage/dissipation), and exponential to the frequency (dynamic/switching activity dissipation). In the case of embedded systems, low-power modes allow parts of the system to be powered down while retaining sufficient functionality to wake on external (and/or internal) events. In sleep mode, both static and dynamic power dissipation is minimized typically to the order of uAmp (in comparison to mAmp in run mode).

  Rust RTIC adopts an event driven approach allowing the system to automatically sleep in case no further tasks are eligible for scheduling. Moreover, leveraging on the zero-cost abstractions in Rust and the guarantees provided by the analysis framework, we do not need to sacrifice correctness/robustness and reliability in order to obtain highly efficient executables.

- Related work:

  In prior work, we developed a tool for automating test case generation and measurements [Hardware-in-the-loop based WCET analysis with KLEE](http://ltu.diva-portal.org/smash/record.jsf?faces-redirect=true&aq2=%5B%5B%5D%5D&af=%5B%5D&searchType=SIMPLE&sortOrder2=title_sort_asc&query=&language=en&pid=diva2%3A1256724&aq=%5B%5B%5D%5D&sf=all&aqe=%5B%5D&sortOrder=author_sort_asc&onlyFullText=false&noOfRows=50&dswid=4983). In this work, automation is done by `gdb` scripting in python (which is tricky due to the internal ad-hoc threading model of `gdb`). In a recent Master's Thesis at LTU [RAUK: Automatic Schedulability Analysis of RTIC Applications Using Symbolic Execution](https://www.diva-portal.org/smash/record.jsf?pid=diva2%3A1652205&dswid=-2492) the tool has been ported to RTIC, and adopting `probe.rs` for automation (replacing `gdb`). The approach taken in RAUK requires certain crates to be overridden by custom counterparts, this unfortunately makes RAUK hard to maintain.

- Future work:

  While the analysis you implemented gives a safe approximation, a more refined analysis is possible by taking resource locking into account when computing interference. This has been further studied in [Enabling Reactive Design of Robust Real-Time Embedded Systems](http://ltu.diva-portal.org/smash/record.jsf?pid=diva2%3A1078928&dswid=-8384).

  Another area for improvements is to extend the RTIC model with formal contracts for the task and resource declarations. This would allow the user to express pre- and post-conditions on tasks and resources, and formally verify their correctness (i.e., that pre-conditions and post-conditions are satisfied for all possible executions). In this way the robustness and reliability for Rust RTIC applications can be further improved.

  Regarding the RTIC framework, we foresee a future development towards improved modularity, facilitating:

  - maintainability, (smaller modules with better separation of concerns)

  - flexibility, (new features can be added and tested in separation)

  - portability, (to targets such as RISC-V and other embedded architectures)

  - third party plugins, (e.g., extending RTIC with proprietary modules for commercial use)

  If you are interested in becoming part of the RTIC development, its an open source project hosted as a github organization [rtic-rs](https://github.com/rtic-rs/), under a permissive MIT/Appache license, open source and free to use.
