// Secrets, so that no generated project starts life with a password someone typed by hand.
// Everything else the template needs is either prompted for or available to Liquid directly,
// such as the built-in {{project-name}}.

let secret_key = "changethis";
let postgres_password = "changethis";

// system::command returns stdout already trimmed. Rhai's own trim() mutates in place and
// returns unit, so calling it here would silently blank the value.
try {
    secret_key = system::command("openssl", ["rand", "-hex", "32"]);
    postgres_password = system::command("openssl", ["rand", "-hex", "24"]);
} catch (err) {
    print("");
    print("  Could not generate secrets automatically: " + err);
    print("  Run `just secrets` in the new project and paste the values into .env.");
    print("");
}

variable::set("secret_key", secret_key);
variable::set("postgres_password", postgres_password);
