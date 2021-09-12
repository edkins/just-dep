# Just Dependent Types

An attempt to produce an interpreted language with dependent types, and otherwise "nothing fancy".

## Values

A value may be:

* an integer
* a string
* an array of values
* a type:
  * `true` or `false`
  * `bool`
  * `int` or `uint`
  * `string`
  * `tuple ts`, where `ts` is a list of types
  * `vector t n`, where `t` is a type and `n` is a uint
  * `list t` where `t` is a type
  * `type`

Note that `true` and `false` are types. `true` has one value: `[]`, and `false` has no values.

## Functions

Mathematically, "functions" are partial functions from the set of values to the set of values. Where there's a gap, we say the function "returns an error".

For now, all functions are computable (excluding the error cases). Each function takes a fixed number of arguments (zero or more) and returns a single value or an error. Errors are returned if the computation does not terminate, or if you call a builtin function that returns an error.

Certain syntactic constructs bypass computation paths. The `if`-`then`-else` statement will only execute one or other of its branches, and the `&&` and `||` operators short-circuit. This means that the overall expression may be a non-error even if one of the subexpressions is an error. There is otherwise no way to "recover" from an error.

Recursive functions are permitted.

## The interpreter

The interpreter's job is to evaluate functions. It contains certain "builtin" functions and can also evaluate any user-defined ones.

The interpreter may panic if it runs out of runtime resources (in particular stack or heap memory). It may also run for a very long time, even for computations that will theoretically terminate.

It may also, of course, return an error if the function is defined to return an error for those arguments.

## Types

Some values are types. There is an "is a" relation, written as `:`, which says whether a given value is of a given type.

Types are used to help the type checker make sure that a given expression will not result in an error. The basic idea is that each expression has an inferred type, and each function has a type signature. Make sure that all the arguments to the function call match the function's type signature, and you know that the resulting expression will have the type given by the function's return type.

Of course in practice it isn't that simple.

* There are subtypes: it's ok to pass a `uint` in to something expecting an `int`
* Types are written as expressions, which may involve unknowns. Is `vector int m` the same as `vector int n`? It depends if `m == n`, which may not be obvious.

## Multiple type signatures for functions

Sometimes it makes sense for a function to have multiple type signatures. A simple example: if you add two integers you can an integer. But if you add two unsigned integers, you get an unsigned integer. The type checker can use the extra information to your advantage.

This is not the same concept as function overloading: the function performs the same operation in both cases. In particular, where the two types overlap, the function needs to return the same answer.

The type checker will try to pick the most specific type signature for the arguments that you give.

## Hidden arguments

Sometimes you want types to be parameterized by something that you don't want to pass in explicitly. When calculating the length of a list, it would be a pain to pass in the type of the list each time.

This language supports a limited form of hidden parameters. Specifically: you cannot use them in the body of the function, where you're calculating the answer. This is to make things simpler for the interpreter: since it doesn't care about type signatures, it can just ignore the hidden parameters and does not have to infer a value for them, which can be nontrivial to do.

They are written within curly brackets.

Hidden arguments may be different for each type signature if a function has multiple of them. For example:

```
length {t:type} (xs:list t) : uint;
length {t:type} {n:uint} (xs:vector t n) : exactly n;
length {ts:list type} (xs:tuple ts) : uint;
```

You can help the type checker along by specifying values for these hidden arguments. The syntax is like so:

```
length {t=int} [1,2,3]
```

In the above case it's ambiguous whether you want the `list` or `vector` version of `length`, since both contain a `t` parameter. This is fine I think? In general, hidden arguments must be labelled with their name and must occur in the correct order relative to each other and to normal arguments.

Note that it's permitted to use your own hidden arguments when filling out hidden arguments. This is because they're ignored by the interpreter.



## Equality

Written as `a == b`, the `equals` function takes a third, hidden, type parameter.

```
equals {t:eq} {a:t} {b:t} : bool;
```

This must be an equivalence relation over each `eq` type. (`eq` is a subtype of type, and represents types over which equality is defined and is computable).

It must also be respected by all functions. That is, if `a == b` then `f a == f b` (and similarly for multi-argument functions).

