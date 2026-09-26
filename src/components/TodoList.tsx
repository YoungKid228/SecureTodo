import { useState } from "react";
import type { TodoItem } from "../App";

interface Props {
  todos: TodoItem[];
  onSave: (todos: TodoItem[]) => void;
}

export default function TodoList({ todos, onSave }: Props) {
  const [text, setText] = useState("");
  const [filter, setFilter] = useState<"all" | "active" | "done">("all");

  function add(e: React.FormEvent) {
    e.preventDefault();
    const t = text.trim();
    if (!t) return;
    const item: TodoItem = {
      id: crypto.randomUUID(),
      text: t,
      done: false,
      created_at: Date.now(),
    };
    onSave([item, ...todos]);
    setText("");
  }

  function toggle(id: string) {
    onSave(todos.map((t) => (t.id === id ? { ...t, done: !t.done } : t)));
  }

  function remove(id: string) {
    onSave(todos.filter((t) => t.id !== id));
  }

  function clearDone() {
    onSave(todos.filter((t) => !t.done));
  }

  const visible = todos.filter((t) =>
    filter === "all" ? true : filter === "active" ? !t.done : t.done
  );
  const activeCount = todos.filter((t) => !t.done).length;

  return (
    <section>
      <form onSubmit={add} className="add-form">
        <input
          placeholder="Новая задача…"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        <button type="submit" disabled={!text.trim()}>
          Добавить
        </button>
      </form>

      <div className="filters">
        {(["all", "active", "done"] as const).map((f) => (
          <button
            key={f}
            className={filter === f ? "chip active" : "chip"}
            onClick={() => setFilter(f)}
          >
            {f === "all" ? "Все" : f === "active" ? "Активные" : "Выполненные"}
          </button>
        ))}
        <span className="muted">Осталось: {activeCount}</span>
        <button className="ghost" onClick={clearDone}>
          Очистить выполненные
        </button>
      </div>

      <ul className="todos">
        {visible.map((t) => (
          <li key={t.id} className={t.done ? "todo done" : "todo"}>
            <input type="checkbox" checked={t.done} onChange={() => toggle(t.id)} />
            <span className="todo-text">{t.text}</span>
            <button className="ghost danger" onClick={() => remove(t.id)} title="Удалить">
              ✕
            </button>
          </li>
        ))}
      </ul>
      {visible.length === 0 && <p className="muted">Пока пусто.</p>}
    </section>
  );
}
