# Just Dependent Types

An attempt to produce a language with dependent types, and otherwise "nothing fancy".

## What is a type?

If two values `x` and `y` are the same type then `x == y` yields either true or false.

If they are different types then `x == y` yields an error.

Intuitively you might think things are "different" if they are different types and hence should compare nonequal. But sometimes, different types can be used to represent the same information, for example `4` vs. `4.0`. If `4 == 4.0` returned false then it would be confusing and might lead to bugs.

"Being the same type" is expected to be an equivalence relation, so the set of all values is partitioned by type.

## What is a format?

A format is concerned with how a given type is stored physically. `u8` and `u16` are both formats of the `int` type. (They are partial in the sense that not all integer values can be represented concretely in that way).

The set of all values is also partitioned by format.

Note that two values can compare equal but have different formats. `4u8 == 4u16` but the format is different.

Formats may refer to runtime parameters by name. Suppose you have a dependently typed function:

```
f: (m:int) -> (n:int) -> int ^ m -> int ^ n -> int
```

This may be given the format:

```
f_concrete: (param0:u64) -> (param0:u64) -> slice u32 param0 -> slice u32 param1 -> u64
```

Note that `slice u32 param0` and `slice u32 param1` are different formats, because they refer to different parameters. But at runtime the types may or may not end up equal, depending on whether the values of m and n are equal. So, formats are known at compile time but types may depend on runtime values.

When a function is compiled, it is compiled based on the formats. Compiling it with the same formats produces the same code, so only needs to be done once and can be cached/reused.

## Format inference

Format can't always be easily inferred. For example, is the value of `(x:u32) + (y:u32)` a `u32` or a `u64`? It depends if we need to make space for it due to overflowing. Put another way, the same function can be compiled with the same argument formats but a different return format.

There will need to be some system of guesswork and hints to get around this problem.

## Omitted constants

One useful kind of format is the omitted constant. This is a constant value which is known at compile time, and so can be omitted at runtime. It doesn't always make sense to use this format if the constant is known, especially if the same function is called with a lot of different constant values.

