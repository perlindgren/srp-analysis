# srp-analysis

This crate provides data structures and algorithms for representing and analyzing Stack Resource Policy (SRP) based task-sets scheduled for singe-core execution.

SRP requires

- Tasks defined by finite set of instructions (i.e., run-to-completion).

- Tasks may request (shared) resources only under LIFO nesting.

SRP comes with a number of outstanding properties:

- Non blocking execution, a task once started is guaranteed all requests to (shared) resources to succeed without blocking.

- Deadlock free execution, no need to worry about deadlock detection/resolution.

- Single blocking, i.e., a task may be blocked from starting by only a _single_ critical section (no transitive blocking possible).

- Shared stack execution, i.e., execution stack is dynamically allocated (from a single shared execution stack) at run-time thus no need to pre-allocate stack for each task.

- Analysis procedures (e.g., task response time and overall schedulability).

SRP supports multi-unit resources and various priority schemes. Here we consider only the special case of single unit resources and static priority scheduling as adopted by the Rust RTIC framework.

## Resources

Start by reading 1, 2 and 3:

1. [A Stack-Based Resource Allocation Policy for Realtime Processes](https://www.math.unipd.it/~tullio/RTS/2009/Baker-1991.pdf), which refers to

2. [Stack-Based Scheduling of Realtime Processes](https://link.springer.com/content/pdf/10.1007/BF00365393.pdf), journal publication based on technical report [3] of the 1991 paper. The underlying model is the same in both papers.

3. [Rate Monotonic Analysis](http://www.di.unito.it/~bini/publications/2003BinButBut.pdf), especially equation 3 is of interest to us.

4. [Hard Real-Time Computing Systems](https://doc.lagout.org/science/0_Computer%20Science/2_Algorithms/Hard%20Real-Time%20Computing%20Systems_%20Predictable%20Scheduling%20Algorithms%20and%20Applications%20%283rd%20ed.%29%20%5BButtazzo%202011-09-15%5D.pdf), Chapter 7, and equation 7.22 specifically for response time analysis.

---

## Definitions

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

In `src/common.rs` a data structure for expressing traces of execution. A number example traces are found in `task_sets`.

---

## Schedulability Analysis

---

### Worst Case Execution Time (WCET)

- `C(t)` the Worst Case Execution Time (WCET) for Task `t`

A trace for a task `t` contains the overall execution time for task `t` as well as the execution time for each (nested) critical section. Traces are assumed input for our analysis.

---

### Total CPU request (aka. total load factor)

Each task `t` has a WCET `C(t)` and given (assumed) inter-arrival time `A(t)`. The CPU request (or load) inferred by a task is `L(t) = C(t) / A(t)`.

We can compute the total CPU request (or load factor), as `Ltot = sum(L(T))`, `T` being the set of tasks `T: {t1..tn}`.

---

### Response time

In general the response time for a task `t` is computed as.

- `R(t) = B(t) + C(t) + I(t)`, where
  - `C(t)` is the task execution time
  - `B(t)` is the blocking time for task `t`, and
  - `I(t)` is the interference (preemptions) to task `t`

For a task set to be schedulable under SRP we have two requirements:

- `Ltot <= 1`. `Ltot > 1` indicates that the total CPU request cannot be met, and thus the system will not be schedulable and further response time analysis is meaningless.
- `R(t) <= D(t)`, for all tasks `t`. `R(t) > D(t)` implies a deadline miss.

---

#### Blocking

SRP brings the outstanding property of single blocking. In other words, a task `t` is blocked by the maximal critical section a task `l` with lower priority (`P(l) < P(t)`) holds a resource `l_r`, with a ceiling `π(l_r)` equal or higher than the priority of `t` (`π(l_r) >= P(t)`).

- `B(t) = max(C(l_r))`, where `P(l) < P(t)`, `π(l_r) >= P(t)`

Implement a function that takes a Task `t` together with the system wide set of tasks `T`, and returns the corresponding blocking time `B(t)`.

---

#### Response time calculation

The response time `R(t)` is also known as the task's _busy period_ `Bp(t)`.

A task `t` may be preempted by each higher priority task `h` (`P(h) > P(t)`) `⌈  Bp(t) / A(h) ⌉` number of times.

We iteratively compute `Bp(t)`, starting from `Bp(t) = B(t) + C(t)` (the initial _busy period_ without interference/preemptions) until a fix-point is reached or the deadline is passed (indicating a non-schedulable system).

---

## Practical considerations for RTIC

The RTIC framework provides both _Hardware_ and _Software_ tasks. _Hardware_ tasks are directly mapped to interrupt vectors with hardware priority set according to the logic priority of the task.

For practical reasons that might imply that two or more tasks (interrupt vectors) will share the same hardware priority (e.g., the Cortex M0/M0+ provides 4 priority levels, the >=M3 typically 8 or 16). In any case we need a way to deal with the case there is a tie in priority. The underlying hardware will in the case of Cortex M based MCUs break the tie by considering the vector index (where lower index takes precedence of higher indexes).

To avoid being tied a specific hardware implementation (like the Cortex M), we propose a worst case estimation by considering unlimited preemption in between interrupts at the same priority. This amounts to response time interference under the condition that `P(h) >= P(t)` in the above. In context of the Cortex M architecture this corresponds to the worst case where `t` is mapped to the vector with the highest vector index, for which `t` will not be allowed to start until all occurrences of `h`, `P(h) >= P(t)` have been dispatched and run-to-completion. This is likely not the exact, as the initial busy period `Bp(t) = B(t) + C(t)` caters the case that we have a preemption _during_ the execution of `t` (but this is not happening here, as interrupts within the same priority will not preempt each other). A tighter bound is thus applied, where we consider interference (without preemption) first and traditional interference (by preemption) later.

Notice, that altering the interference condition `P(h) >= P(t)` in the original recurrence is still a valid over approximation to the problem.

<!-- The SRP analysis assumes tasks to have unique priorities, or as an extension a proper fifo ordering of arrivals on tied priorities. The RTIC framework maps _Hardware_ tasks to interrupts, with static priorities set accordingly. The RTIC framework v1, allows for _Software Tasks_ sharing a single interrupt handler. These will be scheduled by an internal FIFO queue. RTIC v2, does not support internally support task queues, instead co-operative multi-tasking is possible using Rust async/await. In this case the user provides a wait queue, that wakes the corresponding interrupt handler. Analysis of this mechanism is set target for future research.

In the following we focus on _Hardware_ tasks, where we are limited by the set of available interrupt vectors and interrupt priorities, e.g., the Cortex M0/M0+ architecture provides only 32 programmable vectors and 4 levels of static priorities. Whereas 32 tasks is typically sufficient for a lightweight hard real-time application the number of distinct priorities might be insufficient (thus coalescing is required). To this end we assume the given traces to have coalesced priorities matching the underlying hardware limitations. Notice, regarding schedulability and response times, we are not striving for the best schedule here, any schedule that satisfies the requirements is good enough.

To this end we can make a safe (over) approximation, considering tasks at the same priority to interfere with each other. This amounts to computing _busy period_ according to (`P(h) >= P(t)`). A tighter bound may be achieved considering how the underlying hardware resolves priority ties (based on vector index). The approximation caters the worst case, as the interrupt -->
