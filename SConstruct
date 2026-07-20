import subprocess
import sys

env = Environment()


def run(*cmd):
    """Build action that shells out to an external command, streaming its
    output directly and propagating its exit code as a build failure."""

    def action(target, source, env):
        return subprocess.call(list(cmd))

    return action


# `scons web` - run the Sycamore frontend alone in a browser via trunk's dev
# server (hot reload, no Tauri shell).
web = env.Alias("web", [], run("trunk", "serve"))
env.AlwaysBuild(web)

# `scons app` - run the full desktop app in dev mode (Tauri launches trunk's
# dev server itself per beforeDevCommand in tauri.conf.json).
app = env.Alias("app", [], run("cargo", "tauri", "dev"))
env.AlwaysBuild(app)

# `scons build-web` - production build of the frontend only, output to dist/.
build_web = env.Alias("build-web", [], run("trunk", "build", "--release"))
env.AlwaysBuild(build_web)

# `scons build-app` - production build of the full desktop app/installer.
build_app = env.Alias("build-app", [], run("cargo", "tauri", "build"))
env.AlwaysBuild(build_app)

Default(None)

Help("""\
Targets:
  scons web         Run the frontend alone with trunk's dev server (hot reload).
  scons app         Run the full desktop app in dev mode (cargo tauri dev).
  scons build-web   Production build of the frontend only (trunk build --release).
  scons build-app   Production build of the desktop app/installer (cargo tauri build).
""")
