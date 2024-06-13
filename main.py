import re

file = open("church_numerals.lambda").read()


def replace_whitespace(file):
    return "".join(c for c in file if not c.isspace())


def compile(text: str) -> str:
    new_text = ""
    index = 0
    defs = []
    while text[index:].strip().startswith("let"):
        end_index = text[index:].index(";")
        s = text[index:index + end_index].split()
        assert s[0] == "let"
        name = s[1]
        body = "".join(s[2:])
        defs.append((name, body))
        index += end_index + 1
    new_text = text[index:]
    # print(new_text)
    for name, body in reversed(defs):
        new_text = replace_whitespace(f"{name}({new_text}).{body}")
        # print(new_text)
    return new_text


def find_block_end(text: str) -> int:
    # print("called block end with", text)
    stack = 0
    for i, c in enumerate(text):
        if c == "(":
            stack += 1
        elif c == ")":
            stack -= 1
            if stack == 0:
                return i
    raise Exception("Invalid parenthesis")


def parse_args(text: str):
    # print("called args with", text)
    args = []
    index = 0
    while True:
        if index + 1 >= len(text):
            return args
        arg_end = find(text[index+1:], ".")
        block_start = find(text[index+1:], "(")
        if block_start < arg_end:
            arg_end = find_block_end(text[index+1:]) + 1

        if arg_end == float("inf"):
            arg_end = len(text[index+1:])

        arg_text = text[index+1:index+arg_end+1]
        # print(arg_text)
        arg = parse_function(arg_text)
        args.append(arg)
        index += arg_end + 1

    raise Exception("No more than 16 arguments allowed")


def parse_call(text: str):
    # print("called call with", text)
    name_end = text.index(".")
    name = text[:name_end]
    args = parse_args(text[name_end:])
    return name, args


def find(text: str, char: str) -> int | float:
    if char not in text:
        return float("inf")
    return text.index(char)


def parse_function(text: str) -> tuple | str:
    # print("called fun with", text)
    if "(" not in text:
        if "." in text:
            return parse_call(text)
        return text
    if text[0] == "(":
        end = find_block_end(text)
        return parse_function(text[1:end])
    call_name_end = find(text, ".")
    def_name_end = find(text, "(")
    if call_name_end < def_name_end:
        return parse_call(text)
    name_end = def_name_end
    name = text[:name_end]
    # print(text[name_end+1:])
    body_end = find_block_end(text)
    args = []
    body = parse_function(text[name_end+1:body_end])
    if body_end + 1 < len(text) and text[body_end+1] == ".":
        args = parse_args(text[body_end+1:])
    return (name, body, args)


def replace(function, old, new):
    # print(f"called replace({old},{new}) with", function)
    if isinstance(function, str):
        if function == old:
            return new
        return function

    if len(function) == 2:
        # function call
        name, args = function

        assert len(new) == 3, "Can only call functions"
        if name == old:
            # print(f"replacing {old} with {new} and {args}")
            return new[0], new[1], new[2] + [replace(arg, old, new) for arg in args]
        else:
            return name, new[2] + [replace(arg, old, new) for arg in args]

    # function def

    name, body, args = function
    if name == old:
        print("naming conflict", function)
        return function

    new_body = replace(body, old, new)
    return name, new_body, [replace(arg, old, new) for arg in args]


def simplify(function):
    return function
    print("simplify", function)
    if isinstance(function, str):
        return function
    if len(function) == 2:
        # function is not yet resolved
        return function[0], [simplify(arg) for arg in function[1]]

    name, body, args = function
    body = simplify(body)
    if len(args) == 0:
        return name, body, args

    new_function = replace(body, name, args[0])
    if isinstance(new_function, str):
        return new_function
    if len(new_function) == 2:
        return new_function
    return new_function


def interpret_function(function):
    print(function)
    if isinstance(function, str):
        return function
    if len(function) != 3:
        raise Exception("Unresolved function", function[0])

    name, body, args = function
    if len(args) == 0:
        print("Function not called")
        return function

    arg = args[0]
    new_function = replace(body, name, arg)
    if isinstance(new_function, str):
        return new_function
    if len(new_function) == 2:
        print("2", new_function)
        return interpret_function(new_function)
    new_function = (new_function[0], simplify(new_function[1]),
                    new_function[2] + args[1:])
    print("simplified", new_function)
    return interpret_function(new_function)


compiled = compile(file)
print("compiled:    ", compiled)
parsed = parse_function(compiled)
print("parsed:      ", parsed)
print("interpreted: ", interpret_function(parsed))
