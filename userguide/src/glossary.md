# Glossary

## Binding

A recipe to build an instance of a type. It includes dependencies (other types that needs to be
prepared first before the instance can be created), and a way to transform the dependencies into
a new instance, either by a user supplied method or Lockjaw internal generation.

## Module

In all Lockjaw documentations, "module" refers to a *dependency injection module*. When referring to
Rust modules `mod` will be used instead.