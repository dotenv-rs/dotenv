oll
RESULT=${VAR} #value: 'one'

# Curly braces are useful in cases when we need to use a variable with non-alphanumeric name
RESULT=$VAR_2 #value: 'one_2' since $ with no curly braces stops after first non-alphanumeric symbol 
RESULT=${VAR_2} #value: 'two'

# The replacement can be escaped with either single quotes or a backslash:
RESULT='$VAR' #value: '$VAR'
RESULT=\$VAR #value: '$VAR'

# Environment variables are used in the substutution and always override the local variables
RESULT=$PATH #value: the contents of the $PATH environment variable
PATH="My local variable value"
RESULT=$PATH #value: the contents of the $PATH environment variable, even though the local variable is defined
```

Dotenv will parse the file, substituting the variables the way it's described in the comments.


Using the `dotenv!` macro
------------------------------------

Add `dotenv_codegen` to your dependencies, and add the following to the top of
your crate:

```rust
#[macro_use]
extern crate dotenv_codegen;
```

Then, in your crate:

```rust
fn main() {
  println!("{}", dotenv!("MEANING_OF_LIFE"));
}
```

[dotenv]: https://github.com/bkeepers/dotenv
