# Preposterous (`prep`)

Preposterous is a language-agnostic preprocessor, similar to the built-in C
preprocessor.

Here's a simple example with file inclusion, basic string manipulation, and
variable substitution.
```c
$PREP enable
// main.c

$$ `stringify` will automatically escape quotation marks, but if you need to
$PREP stringify LOREM lorem.txt
$PREP define IPSUM "consectetur \%adipiscing\% elit"
$PREP concat LOREMIPSUM %LOREM%, %IPSUM%

$PREP include print.c

int main() {
    print(%LOREMIPSUM%);

    return 0;
}
```

```c
// print.c (same directory as main.c)
// there's no `$PREP enable` line, so this won't get processed

#include <stdio.h>

void print(char *text) {
    printf("I say: %s\n", text);
}
```

```
$PREP enable
$$ lorem.txt
Lorem "ipsum" dolor sit amet
```

The result should look like this:

```c
// main.c



// print.c (same directory as main.c)
// there's no `$PREP enable` line, so this won't get processed

#include <stdio.h>

void print(char *text) {
    printf("I say: %s\n", text);
}


int main() {
    print("Lorem \"ipsum\" dolor sit amet, consectetur %adipiscing% elit ");

    return 0;
}
```

Here's the full list of macros:
- `enable`: Enables preprocessing of the file. Must be the first line.
- `disable`: Prevents preprocessing of the file. Must be the first line.
- `$$...`: Comment. Must be at the start of the line.
- `include <file>`: Copy-pastes the contents of the file (with preprocessing).
- `define <name> <...>`: Define a preprocessor variable.
  The arguments are strictly treated as a string.
- `concat <name> <args...>`: Concatenate multiple arguments. De-stringifies
  each one, then re-stringifies the whole thing.
- `stringify <name> <file>`: Loads the contents of the file into a variable,
  performing the following manipulations:
    - Quoting (by default, `"<content>"`, configure with
      `quotes "<opening>" "<closing>"`)
    - Quote escaping (with backslashes, disable with `no_quote_escape`,
      configure with `quote_escape "<escape>"`)
    - Newline escaping (with backslashes, disable with `no_newline_escape`,
      configure with `newline_escape "<escape>"`)

And some for post-1.0:
- `defun (<args...>) "<body>"`: Define a "function".
- `call <function> <args...>`: Call a defined function.

With lots more to come.