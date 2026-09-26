import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import LockScreen from "./components/LockScreen";
import TodoList from "./components/TodoList";
import ChangePassword from "./components/ChangePassword";
import "./App.css";

export interface TodoItem {
  id: string;
  text: string;
  done: boolean;
  created_at: number;
}

interface VaultStatus {
  initialized: boolean;
  unlocked: boolean;
}

export default function App() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [todos, setTodos] = useState<TodoItem[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showChangePw, setShowChangePw] = useState(false);

  async function refresh() {
    try {
      const s = await invoke<VaultStatus>("vault_status");
      setStatus(s);
      if (s.unlocked) {
        const items = await invoke<TodoItem[]>("todos_list");
        setTodos(items);
      } else {
        setTodos([]);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleAuth(password: string, mode: "unlock" | "init") {
    setBusy(true);
    setError(null);
    try {
      if (mode === "init") {
        const s = await invoke<VaultStatus>("vault_init", { password });
        setStatus(s);
      } else {
        const s = await invoke<VaultStatus>("vault_unlock", { password });
        setStatus(s);
      }
      const items = await invoke<TodoItem[]>("todos_list");
      setTodos(items);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleLock() {
    setBusy(true);
    try {
      const s = await invoke<VaultStatus>("vault_lock");
      setStatus(s);
      setTodos([]);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleSave(next: TodoItem[]) {
    setTodos(next);
    setError(null);
    try {
      await invoke("todos_save", { todos: next });
    } catch (e) {
      setError(String(e));
    }
  }

  if (!status) {
    return (
      <main className="container">
        <p>Загрузка…</p>
      </main>
    );
  }

  if (!status.unlocked) {
    return (
      <main className="container">
        <LockScreen
          initialized={status.initialized}
          busy={busy}
          error={error}
          onAuth={handleAuth}
        />
      </main>
    );
  }

  return (
    <main className="container">
      <header className="header">
        <h1>🔒 Задачи</h1>
        <div className="header-actions">
          <button className="ghost" onClick={() => setShowChangePw((v) => !v)}>
            Сменить пароль
          </button>
          <button className="ghost" onClick={handleLock} disabled={busy}>
            🔒 Заблокировать
          </button>
        </div>
      </header>
      {error && <div className="error">{error}</div>}
      {showChangePw && (
        <ChangePassword
          busy={busy}
          onDone={(ok, msg) => {
            if (msg) setError(msg);
            else setError(null);
            if (ok) setShowChangePw(false);
          }}
          setBusy={setBusy}
        />
      )}
      <TodoList todos={todos} onSave={handleSave} />
    </main>
  );
}
