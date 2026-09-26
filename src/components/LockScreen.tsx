import { useState } from "react";

interface Props {
  initialized: boolean;
  busy: boolean;
  error: string | null;
  onAuth: (password: string, mode: "unlock" | "init") => void;
}

export default function LockScreen({ initialized, busy, error, onAuth }: Props) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  const mode: "unlock" | "init" = initialized ? "unlock" : "init";

  function submit(e: React.FormEvent) {
    e.preventDefault();
    if (mode === "init" && password !== confirm) return;
    onAuth(password, mode);
  }

  return (
    <div className="card">
      <h1>🔐 Secure Todo</h1>
      <p className="muted">
        {mode === "init"
          ? "Создайте пароль. Он нигде не хранится — только его хэш для проверки, а задачи шифруются и расшифровываются только после ввода пароля."
          : "Введите пароль, чтобы расшифровать задачи."}
      </p>
      <form onSubmit={submit} className="form">
        <input
          type="password"
          placeholder={mode === "init" ? "Новый пароль (мин. 4 символа)" : "Пароль"}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoFocus
        />
        {mode === "init" && (
          <input
            type="password"
            placeholder="Повторите пароль"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        )}
        {mode === "init" && password !== confirm && confirm.length > 0 && (
          <div className="error">Пароли не совпадают</div>
        )}
        <button
          type="submit"
          disabled={busy || password.length === 0 || (mode === "init" && password !== confirm)}
        >
          {busy ? "…" : mode === "init" ? "Создать хранилище" : "Разблокировать"}
        </button>
      </form>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
