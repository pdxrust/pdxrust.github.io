+++
title = "Toasty: An Easy-to-use Object-Relational Mapping for Rust"
start = 2026-01-08T19:00:00-08:00
kind = "talk"
status = "scheduled"

[location]
name = "Portland State University, Engineering Building, room EB 102"
address = "1930 SW 4th Ave, Portland, OR"

[[speakers]]
name = "Carl Lerche"
+++

An object-relational mapping (ORM) is a layer that lets you work with a database
using native types instead of writing queries directly. In Rust terms, an ORM
maps tables to structs, rows to struct instances, and columns to fields, handling
serialization, deserialization, and query construction for you.

Toasty is an experimental new ORM for Rust that prioritizes ease of use. It
supports both SQL databases as well as some NoSQL databases, including DynamoDB
and Cassandra. Rather than hiding the database's capabilities, Toasty exposes
features based on the target database.

Carl Lerche, Toasty's designer, is also the original designer of the Tokio
asynchronous runtime library.
