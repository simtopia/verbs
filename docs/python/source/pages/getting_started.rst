***************
Getting Started
***************

Building from Source
====================

Requirements
------------

Building verbs requires a rust and cargo installation
see `here <https://doc.rust-lang.org/cargo/getting-started/installation.html>`_
for instructions.

Verbs uses `hatch <https://hatch.pypa.io/latest/>`_ for dependency control,
see the `hatch docs <https://hatch.pypa.io/latest/install/>`_ for installation
instructions.

Building
--------

Using hatch the package can be built by running

.. code-block::

   hatch run dev:build

from the repo root, which will build the package to ``target/``

Jupyter Notebook
----------------

If you want to start a Jupyter Notebook with verbs installed you
can run

.. code-block::

   hatch run notebook:jupyter

which will build the package and start a jupyter server.
