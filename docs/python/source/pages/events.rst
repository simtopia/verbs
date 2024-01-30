*************
Events & Logs
*************

The simulation environment stores events
generated during the update/processing of
a block, and the history of events generated
over the course.

Data from events is wrapped with additional
data on the step and function that generated
the event to allow agents to filter events
and the history of events to be reconstructed.

Events are represented in Python by a tuple
containing:

* The 4 byte function selector that generated
  the events.
* A list of events/logs.
* The simulation step the event was generated
  in.
* The sequence inside the block the event
  occurred, i.e. these values order the
  events inside a block.

The list of events is itself a list of tuples
containing the address of the contract and
the corresponding event data.

The events generated in the last simulated block
can be retrieved using the
:py:meth:`verbs.envs.EmptyEnv.get_last_events`
and the full history of events over the course
of the simulation with
:py:meth:`verbs.envs.EmptyEnv.get_event_history`.

Data from events can then be decoded using the
:py:meth:`verbs.abi.Event.decode` method.
