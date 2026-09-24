# omikuji-proton-monkey-patch
import os
import sys

_PIN_VAR = "OMIKUJI_PROTON_DLLS_PIN"
_SKIP_VAR = "OMIKUJI_PROTON_DLLS_SKIP"
_SYSTEM_DIRS = ("system32", "syswow64")


def _log(line):
    try:
        main = sys.modules.get("__main__")
        emit = getattr(main, "log", None)
        if callable(emit):
            emit("omikuji: " + str(line))
        else:
            sys.stderr.write("omikuji: " + str(line) + "\n")
    except Exception:
        pass


class _PinnedOverrides(dict):
    def __init__(self, base, pinned):
        super().__init__(base)
        self._pinned = pinned
        super().update(pinned)

    def __setitem__(self, key, value):
        if key in self._pinned:
            _log("holding %s=%s, refused %s" % (key, self._pinned[key], value))
            return
        super().__setitem__(key, value)

    def __delitem__(self, key):
        if key in self._pinned:
            return
        try:
            super().__delitem__(key)
        except KeyError:
            pass

    def update(self, *args, **kwargs):
        for key, value in dict(*args, **kwargs).items():
            self[key] = value


def _names(raw):
    return [name.strip() for name in raw.split(",") if name.strip()]


def _parse(raw):
    pinned = {}
    for entry in raw.split(";"):
        entry = entry.strip()
        if not entry or "=" not in entry:
            continue
        names, setting = entry.rsplit("=", 1)
        for name in _names(names):
            pinned[name] = setting.strip()
    return pinned


def _into_system_dir(dst):
    parts = os.path.normpath(str(dst)).lower().split(os.sep)
    return any(d in parts for d in _SYSTEM_DIRS)


def _install_pins(main):
    raw = os.environ.get(_PIN_VAR, "").strip()
    if not raw:
        return

    session = getattr(main, "g_session", None)
    current = getattr(session, "dlloverrides", None)
    if not isinstance(current, dict):
        _log("this proton exposes no dll override table, translation layer toggles will not apply")
        return

    pinned = _parse(raw)
    if not pinned:
        return

    session.dlloverrides = _PinnedOverrides(current, pinned)
    _log("pinning %s" % ", ".join("%s=%s" % pair for pair in sorted(pinned.items())))


def _install_skips(main):
    skipped = {name.lower() + ".dll" for name in _names(os.environ.get(_SKIP_VAR, ""))}
    if not skipped:
        return

    original = getattr(main, "try_copy", None)
    if not callable(original):
        _log("this proton has no try_copy, translation layer versions will not apply")
        return

    def try_copy(src, dst, *args, **kwargs):
        if os.path.basename(str(src)).lower() in skipped and _into_system_dir(dst):
            return
        return original(src, dst, *args, **kwargs)

    main.try_copy = try_copy
    _log("leaving %s to omikuji" % ", ".join(sorted(skipped)))


for _install in (_install_pins, _install_skips):
    try:
        _install(sys.modules.get("__main__"))
    except Exception as exc:
        _log("%s failed, proton keeps its own settings (%r)" % (_install.__name__, exc))

user_settings = {}
