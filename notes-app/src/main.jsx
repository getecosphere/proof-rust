import React, { useEffect, useMemo, useRef, useState, useCallback } from "react";
import { createRoot } from "react-dom/client";
import { motion, AnimatePresence } from "motion/react";
import styles from "./styles.css?inline";

const spring = { type: "spring", stiffness: 520, damping: 42, mass: 0.9 };
const soft = { type: "spring", stiffness: 300, damping: 30, mass: 0.8 };

function getSession() {
  try {
    const s = JSON.parse(localStorage.getItem("eco_session") || "null");
    return s && s.token ? s : null;
  } catch {
    return null;
  }
}

function esc(v) {
  return String(v ?? "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function relTime(iso) {
  if (!iso) return "";
  const t = new Date(iso);
  const diff = (Date.now() - t.getTime()) / 1000;
  if (diff < 60) return "just now";
  if (diff < 3600) return Math.floor(diff / 60) + "m ago";
  if (diff < 86400) return Math.floor(diff / 3600) + "h ago";
  return t.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

const api = (token) => async (path, opts = {}) => {
  const res = await fetch(path, {
    ...opts,
    headers: {
      Authorization: "Bearer " + token,
      ...(opts.body ? { "Content-Type": "application/json" } : {}),
      ...(opts.headers || {}),
    },
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || res.statusText || res.status);
  return data;
};

function Bell({ token }) {
  const [open, setOpen] = useState(false);
  const [count, setCount] = useState(0);
  const [items, setItems] = useState(null);
  const request = useMemo(() => api(token), [token]);

  useEffect(() => {
    const poll = () =>
      request("/api/notifications/unread-count")
        .then((d) => setCount(d.unread_count || 0))
        .catch(() => {});
    poll();
    const t = setInterval(poll, 30000);
    return () => clearInterval(t);
  }, [request]);

  const load = useCallback(() => {
    request("/api/notifications")
      .then((list) => {
        setItems(list || []);
        setCount(0);
      })
      .catch(() => setItems([]));
  }, [request]);

  useEffect(() => {
    if (open) load();
  }, [open, load]);

  return (
    <div className="bell-wrap">
      <motion.button
        className="bell"
        aria-label="Notifications"
        onClick={() => {
          setOpen((o) => !o);
          if (!open) load();
        }}
        whileHover={{ scale: 1.06 }}
        whileTap={{ scale: 0.92 }}
      >
        <BellIcon />
        <AnimatePresence>
          {count > 0 && (
            <motion.span
              key="badge"
              className="badge"
              initial={{ scale: 0.4, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.4, opacity: 0 }}
              transition={spring}
            >
              {count > 99 ? "99+" : count}
            </motion.span>
          )}
        </AnimatePresence>
      </motion.button>
      <AnimatePresence>
        {open && (
          <motion.div
            className="bell-panel"
            initial={{ opacity: 0, y: -8, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -6, scale: 0.97 }}
            transition={spring}
          >
            <div className="bell-head">Notifications</div>
            <div className="bell-list">
              {items === null && <p className="bell-empty">Loading…</p>}
              {items && items.length === 0 && <p className="bell-empty">No notifications yet.</p>}
              {items &&
                items.map((n) => (
                  <motion.div
                    key={n.id}
                    className={"bell-item" + (n.read ? "" : " unread")}
                    layout
                    initial={{ opacity: 0, y: 6 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={soft}
                  >
                    <div className="bell-title">{esc(n.title)}</div>
                    <div className="bell-body">{esc(n.body || "")}</div>
                    <div className="bell-time">{esc(relTime(n.createdAt))}</div>
                  </motion.div>
                ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function BellIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"></path>
      <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
    </svg>
  );
}

function NotesApp() {
  const session = useMemo(getSession, []);
  if (!session) {
    window.location.href = "/signin";
    return null;
  }
  return <Workspace token={session.token} name={session.user?.name || "there"} />;
}

function Workspace({ token, name }) {
  const request = useMemo(() => api(token), [token]);
  const [notes, setNotes] = useState(null);
  const [query, setQuery] = useState("");
  const [current, setCurrent] = useState(null);
  const [draft, setDraft] = useState({ title: "", body: "", pinned: false });
  const [dirty, setDirty] = useState(false);
  const saveTimer = useRef(null);

  const load = useCallback(() => {
    request("/api/notes")
      .then((list) => {
        setNotes(list || []);
        if (list && list.length && !current) setCurrent(list[0].id);
      })
      .catch(() => setNotes([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [request]);

  useEffect(load, [load]);

  const currentNote = useMemo(() => notes?.find((n) => n.id === current) || null, [notes, current]);
  const filtered = useMemo(() => {
    if (!notes) return [];
    const q = query.trim().toLowerCase();
    if (!q) return notes;
    return notes.filter(
      (n) =>
        (n.title || "").toLowerCase().includes(q) ||
        (n.body || "").toLowerCase().includes(q)
    );
  }, [notes, query]);

  useEffect(() => {
    if (!currentNote) return;
    setDraft({ title: currentNote.title || "", body: currentNote.body || "", pinned: !!currentNote.pinned });
    setDirty(false);
  }, [currentNote]);

  const save = useCallback(
    (force) => {
      if (!current || !(force || dirty)) return;
      request(`/api/notes/${encodeURIComponent(current)}`, {
        method: "PUT",
        body: JSON.stringify(draft),
      })
        .then((updated) => {
          setNotes((prev) => (prev || []).map((n) => (n.id === updated.id ? updated : n)));
          setDirty(false);
        })
        .catch(() => {});
    },
    [request, current, dirty, draft]
  );

  useEffect(() => {
    if (!dirty) return;
    saveTimer.current = setTimeout(() => save(true), 2200);
    return () => clearTimeout(saveTimer.current);
  }, [dirty, draft, save]);

  const create = () => {
    request("/api/notes", { method: "POST", body: JSON.stringify({ title: "", body: "" }) }).then((n) => {
      setNotes((prev) => [n, ...(prev || [])]);
      setCurrent(n.id);
      setQuery("");
    });
  };

  const remove = () => {
    if (!current) return;
    request(`/api/notes/${encodeURIComponent(current)}`, { method: "DELETE" })
      .then(() => {
        const rest = (notes || []).filter((n) => n.id !== current);
        setNotes(rest);
        setCurrent(rest.length ? rest[0].id : null);
        setDirty(false);
      })
      .catch(() => {});
  };

  const initials = (name || "?")
    .split(/\s+/)
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <div className="ns">
      <style>{styles}</style>
      <header className="ns-toolbar">
        <motion.div className="ns-avatar" initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }} transition={spring}>
          {esc(initials)}
        </motion.div>
        <motion.div className="ns-name" initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }} transition={{ ...spring, delay: 0.04 }}>
          <b>{esc(name)}</b>
          <span>Notes — MongoDB-backed, motion-driven</span>
        </motion.div>
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.08 }}>
          <Bell token={token} />
        </motion.div>
      </header>

      <div className="ns-body">
        <motion.aside
          className="ns-sidebar"
          initial={{ x: -24, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ ...spring, delay: 0.05 }}
        >
          <div className="ns-side-head">
            <span className="ns-side-title">Notes</span>
            <motion.button className="ns-new" onClick={create} whileHover={{ scale: 1.08 }} whileTap={{ scale: 0.88 }} aria-label="New note">
              +
            </motion.button>
          </div>
          <input
            className="ns-search"
            placeholder="Search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <div className="ns-list">
            <AnimatePresence mode="popLayout">
              {filtered.length === 0 && (
                <motion.p key="empty" className="ns-empty" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
                  {notes === null ? "Loading…" : query ? "No results" : "No Notes"}
                </motion.p>
              )}
              {filtered.map((n) => {
                const active = current === n.id;
                const snippet = (n.body || "").replace(/\s+/g, " ").trim();
                return (
                  <motion.button
                    key={n.id}
                    className={"ns-row" + (active ? " active" : "")}
                    layout
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.96, height: 0, marginBottom: 0 }}
                    transition={spring}
                    onClick={() => setCurrent(n.id)}
                    whileTap={{ scale: 0.985 }}
                  >
                    {active && <motion.span layoutId="ns-active-bg" className="ns-active-bg" transition={spring} />}
                    <span className="ns-row-title">
                      {n.pinned ? "📌 " : ""}
                      {esc(n.title || "New Note")}
                    </span>
                    <span className="ns-row-meta">
                      {esc(relTime(n.updatedAt))}
                      {snippet ? " · " + esc(snippet.slice(0, 58)) : ""}
                    </span>
                  </motion.button>
                );
              })}
            </AnimatePresence>
          </div>
        </motion.aside>

        <motion.section
          className="ns-editor"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.1 }}
        >
          {!currentNote ? (
            <div className="ns-editor-empty">
              <motion.p initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={soft}>
                Select a note, or create a new one.
              </motion.p>
            </div>
          ) : (
            <motion.div key={current} initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={soft} className="ns-editor-inner">
              <div className="ns-editor-top">
                <input
                  className="ns-title"
                  placeholder="Title"
                  value={draft.title}
                  onChange={(e) => {
                    setDraft((d) => ({ ...d, title: e.target.value }));
                    setDirty(true);
                  }}
                />
                <motion.span
                  className="ns-savedot"
                  animate={{ opacity: dirty ? 1 : 0, scale: dirty ? 1 : 0.6 }}
                  transition={spring}
                />
                <label className="ns-pin">
                  <input
                    type="checkbox"
                    checked={draft.pinned}
                    onChange={(e) => {
                      setDraft((d) => ({ ...d, pinned: e.target.checked }));
                      setDirty(true);
                    }}
                  />
                  <span>Pin</span>
                </label>
                <motion.button className="ns-del" onClick={remove} whileTap={{ scale: 0.92 }}>
                  Delete
                </motion.button>
              </div>
              <textarea
                className="ns-body"
                placeholder="Start writing…"
                value={draft.body}
                onChange={(e) => {
                  setDraft((d) => ({ ...d, body: e.target.value }));
                  setDirty(true);
                }}
              />
              <div className="ns-hint">Autosaves · ⌘S to save now · every note belongs to you (MongoDB)</div>
            </motion.div>
          )}
        </motion.section>
      </div>
    </div>
  );
}

const el = document.getElementById("eco-notes-root");
if (el) createRoot(el).render(<NotesApp />);
