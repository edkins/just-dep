# Just Dependent Types

An attempt to produce an interpreted language with dependent types, and otherwise "nothing fancy".

## Values

A value may be:

* an integer
* f32
* f64
* true or false
* an error
* a list of values
* a function, that takes a value as an argument and returns a value
* (user-defined structures to be determined)

## Equality

The `==` relation returns true, false or an error.

* Integers compare equal if they're equal
* f32 and f64 cannot be compared for equality
* true, false compare equal if they're equal
* errors propagate through `==`
* lists compare equal if they are of the same length and all they compare equal elementwise.
* lists compare nonequal if they are of different length
* lists compare nonequal if the first few elements compare equal and the next compares nonequal (even if the remainder would give errors).
* functions cannot be compared for equality
* (user-defined structures to be determined)
* comparing elements across these categorizations returns an error, e.g. comparing an integer against true.

As you can see, `==` only acts as an equivalence relation within certain domains. When used more broadly than that, it might return an error instead. Examples where it behaves sensibly:

* across `int`
* across `bool`
* across `tuple[A,B]` where it behaves sensibly across A and B
* across `A^n` where it behaves sensibly across A, and A is a nonnegative integer
* across `A*` where it behaves sensibly across A

These can be thought of as "types", but a given value may belong to multiple of them. For instance, `[1,2]` belongs to `tuple[int,int]`, `int^2` and `int*`.

## Function signatures

Function signatures contain things that look like types.

```
f: int -> int
```

What this means is:

* If you call `f` with a value that isn't an `int`, it will return an error (i.e. a precondition)
* You're asserting that otherwise, `f` will always return an `int` (i.e. a postcondition).

The first can be checked at runtime, but the second cannot. We need a _proof_ that the function actually yields the desired value.

Note that all that is required on these "types" is that they are predicates, i.e. they can decide membership of any given value. They don't have to behave nicely with `==`.

(TODO: decide whether any predicate can be used here).
