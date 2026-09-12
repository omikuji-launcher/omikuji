# omikuji-proton-monkey-patch 1
import os
import sys

_PIN_VAR = "OMIKUJI_PROTON_DLLS_PIN"


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


def _parse(raw):
    pinned = {}
    for entry in raw.split(";"):
        entry = entry.strip()
        if not entry or "=" not in entry:
            continue
        names, setting = entry.rsplit("=", 1)
        for name in names.split(","):
            name = name.strip()
            if name:
                pinned[name] = setting.strip()
    return pinned


def _install():
    raw = os.environ.get(_PIN_VAR, "").strip()
    if not raw:
        return

    session = getattr(sys.modules.get("__main__"), "g_session", None)
    current = getattr(session, "dlloverrides", None)
    if not isinstance(current, dict):
        _log("this proton exposes no dll override table, translation layer toggles will not apply")
        return

    pinned = _parse(raw)
    if not pinned:
        return

    session.dlloverrides = _PinnedOverrides(current, pinned)
    _log("pinning %s" % ", ".join("%s=%s" % pair for pair in sorted(pinned.items())))


try:
    _install()
except Exception as exc:
    _log("monkey patch failed, proton keeps its own settings (%r)" % (exc,))

user_settings = {}
