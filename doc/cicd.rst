CI testing
==========

#. Run static checkers
#. Build everything:

   - Python package

#. Upload the Python package to a local package index.
#. Run tests from both source dir and installed package.
#. Upload the Python package to test.pypi.org and pypi.org, if needed.
#. Prune old and vulnerable packages from the package index.
