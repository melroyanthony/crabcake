// The template's own README stays behind on GitHub; the project gets its own.
file::rename("README.project.md", "README.md");

let name = variable::get("project-name");

print("");
print("  " + variable::get("project_title") + " is ready.");
print("");
print("    cd " + name);
print("    just up          # start postgres, the api, the worker and the frontend");
print("    just client      # regenerate the typed api client after changing the api");
print("");
print("  Frontend  http://localhost:3000");
print("  API docs  http://localhost:8000/docs");
print("  Mail      http://localhost:1080");
print("");
print("  Secrets in .env were generated for you. Review them before deploying, and");
print("  keep .env out of any public repository.");
print("");
