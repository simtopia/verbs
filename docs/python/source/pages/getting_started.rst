***************
Getting Started
***************

Installation
============

Verbs can be installed via pip

.. code-block:: bash

   pip install verbs

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

.. note::

   OSX users may need to install `patchelf` with `homebrew <https://brew.sh>`_
   using

   .. code-block:: bash

      brew install patchelf

Building
--------

Using hatch the package can be built by running

.. code-block:: bash

   hatch run dev:build

from the repo root, which will build the package to ``target/``

Jupyter Notebook
----------------

If you want to start a Jupyter Notebook with verbs installed you
can run

.. code-block:: bash

   hatch run notebook:jupyter

which will build the package and start a jupyter server.

Github Dependency
-----------------

Verbs can be added as a github dependency in the `pyproject.toml`

.. code-block:: bash

   dependencies = [
      "verbs@git+ssh://git@github.com/simtopia/verbs.git"
   ]

but also requires that maturin is added as a build requirement,
for example

.. code-block:: bash

   [build-system]
   requires = ["setuptools >= 61.0", "maturin>=1.2,<2.0"]
   build-backend = "setuptools.build_meta"
