
# Christofer Nolander's Portfolio

This is the repository containing the source code for the server and website
powering my [portfolio](nolander.me). 


## Design

The `pages` directory contains the actual pages making up the website. Each
subdirectory contains a file `index.yml` which describes the content of that
specific page. Pages are rendered using a templating library called Handlebars
which gets it's data from the file pointed to by `index.yml`.

The `server` directory has the source code for the server. The server is general
in its design and compiles down to a single binary which provides a Command Line
Interface (CLI). The CLI allows the server to be configured to fit the needs of
the host environemnt. The server supports watching the `pages` and `templates`
directories for changes and updating the website's contents in such an event.
This is achieved by hot-swapping the server's configuration using atomic
operations.

`templates` has some general purpose templates which are used across the site
(ie. the sidebar and HTML metadata).


## Server Features

- Support for precompiled Markdown
- Hot reloading of website data
- HTML templating
- Configurable

