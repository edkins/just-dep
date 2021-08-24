# Just Dependent Types

An attempt to produce an interpreted language with dependent types, and otherwise "nothing fancy".

## Values

A value may be:

* true or false
* an integer
* an error
* a list of values
* a type:
  * `bool`
  * `int` or `uint`
  * `tuple ts`, where `ts` is a list of types
  * `t ^ n`, where `t` is a type and `n` is a uint
  * `t*` where `t` is a type
  * `type`
  * note that `true` and `false` are also types. `true` has one value: `[]`, and `false` has no values.

## Equality

The `==` relation returns true, false or an error.

* Integers compare equal if they're equal
* true, false compare equal if they're equal
* errors propagate through `==`
* lists compare equal if they are of the same length and all they compare equal elementwise.
* lists compare nonequal if they are of different length
* lists compare nonequal if the first few elements compare equal and the next compares nonequal (even if the remainder would give errors).
* types compare equal if they're written the same, with the special case that `t^n == tuple (replicate n ts)`
* comparing elements across these categorizations returns an error, e.g. comparing an integer against true.

As you can see, `==` only acts as an equivalence relation within certain domains. When used more broadly than that, it might return an error instead. It generally behaves sensibly across any given type though.

## Function signatures

Function types look like this:

```
f: int -> int
```

What this means is:

* If you call `f` with a value that isn't an `int`, it will return an error (i.e. a precondition)
* You're asserting that otherwise, `f` will always return an `int` (i.e. a postcondition).

To start with, these will just be checked at runtime.

