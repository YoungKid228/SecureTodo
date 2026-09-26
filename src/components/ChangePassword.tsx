import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  busy: boolean;
  setBusy: (v: boolean) => void;
  onDone: (ok: boolean, msg: string | null) => void;
}

export default function ChangePassword({ busy, setBusy, onDone }: Props) {
  const [oldPw, setOldPw] = useState("");
  const [newPw, setNewPw] = useState("");
  const [confirm, setConfirm] = useState("");

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (newPw !== confirm) return;
    setBusy(true);
    try {
      await invoke("vault_change_password", { oldPassword: oldPw, newPassword: newPw });
      setOldPw("");
      setNewPw("");
      setConfirm("");
      onDone(true, null);
    } catch (err) {
      onDone(false, String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit} className="card small">
      <h3>Смена пароля</h3>
      <input
        type="password"
        placeholder="Старый пароль"
        value={oldPw}
        onChange={(e) => setOldPw(e.target.value)}
      />
      <input
        type="password"
        placeholder="Новый пароль (мин. 4 символа)"
        value={newPw}
        onChange={(e) => setNewPw(e.target.value)}
      />
      <input
        type="password"
        placeholder="Повторите новый пароль"
        value={confirm}
        onChange={(e) => setConfirm(e.target.value)}
      />
      {newPw !== confirm && confirm.length > 0 && (
        <div className="error">Пароли не совпадают</div>
      )}
      <button type="submit" disabled={busy || !oldPw || !newPw || newPw !== confirm}>
        {busy ? "…" : "Сменить пароль"}
      </button>
    </form>
  );
}
