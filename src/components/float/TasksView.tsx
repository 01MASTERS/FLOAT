import React, { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Check, X } from "lucide-react";
import { Task, taskList, useTasks } from "../../activities/tasksStore";
import "./TasksView.css";

/** To-do list: type, Enter, tick off. Open tasks first, finished ones below. */
export const TasksView: React.FC = () => {
  const tasks = useTasks();
  const [draft, setDraft] = useState("");
  const open = tasks.filter((t) => !t.done);
  const done = tasks.filter((t) => t.done);

  const [editingTaskId, setEditingTaskId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [shakeTaskId, setShakeTaskId] = useState<string | null>(null);
  const shakeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (shakeTimerRef.current) clearTimeout(shakeTimerRef.current);
    };
  }, []);
  const editInputRef = useRef<HTMLInputElement>(null);

  const startEdit = (task: Task) => {
    setEditingTaskId(task.id);
    setEditTitle(task.title);
    setTimeout(() => {
      if (editInputRef.current) {
        editInputRef.current.focus();
        editInputRef.current.select();
      }
    }, 0);
  };

  const cancelEdit = () => {
    setEditingTaskId(null);
    setEditTitle("");
  };

  const commitEdit = (task: Task) => {
    const trimmed = editTitle.trim().slice(0, 200);
    if (!trimmed) {
      setShakeTaskId(task.id);
      if (shakeTimerRef.current) clearTimeout(shakeTimerRef.current);
      shakeTimerRef.current = setTimeout(() => setShakeTaskId(null), 400);
      cancelEdit();
      return;
    }
    if (trimmed !== task.title) {
      taskList.updateTitle(task.id, trimmed);
    }
    cancelEdit();
  };

  const handleEditKeyDown = (e: React.KeyboardEvent<HTMLInputElement>, task: Task) => {
    e.stopPropagation();
    if (e.key === "Enter") {
      e.preventDefault();
      commitEdit(task);
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEdit();
    }
  };

  return (
    <div className="view tasks-view">
      <div className="view-header">
        <span className="view-title">Tasks</span>
        <span className="view-hint">{tasks.length === 0 ? "" : open.length === 0 ? "All done" : `${open.length} to do`}</span>
      </div>

      <form
        className="tasks-composer"
        onSubmit={(e) => {
          e.preventDefault();
          const clean = draft.trim();
          if (!clean) return;
          taskList.add(clean);
          setDraft("");
        }}
      >
        <input
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="Add a task… (Enter)"
          maxLength={120}
        />
      </form>

      <div className="tasks-list scroll-area">
        <AnimatePresence initial={false}>
          {open.map((t) => {
            const isEditing = editingTaskId === t.id;
            const isShaking = shakeTaskId === t.id;

            return (
              <motion.div
                key={t.id}
                className={`task-row ${isShaking ? "task-shake" : ""}`}
                layout
                initial={{ opacity: 0, y: -6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95 }}
                transition={{ duration: 0.18, ease: "easeOut" }}
              >
                <button
                  type="button"
                  className="task-check"
                  onClick={() => taskList.toggle(t.id)}
                  aria-label="Mark done"
                />
                {isEditing ? (
                  <input
                    ref={editInputRef}
                    className="task-inline-edit"
                    value={editTitle}
                    onChange={(e) => setEditTitle(e.target.value)}
                    onBlur={() => commitEdit(t)}
                    onKeyDown={(e) => handleEditKeyDown(e, t)}
                    maxLength={200}
                  />
                ) : (
                  <span
                    className="task-title"
                    onDoubleClick={() => startEdit(t)}
                    title="Double-click to edit"
                  >
                    {t.title}
                  </span>
                )}
                <button
                  type="button"
                  className="task-delete"
                  onClick={() => taskList.remove(t.id)}
                  aria-label="Delete task"
                >
                  <X size={12} />
                </button>
              </motion.div>
            );
          })}
        </AnimatePresence>

        {done.length > 0 && (
          <div className="tasks-done-group">
            <span className="tasks-done-header">Completed</span>
            {done.map((t) => (
              <div key={t.id} className="task-row is-done">
                <button
                  type="button"
                  className="task-check checked"
                  onClick={() => taskList.toggle(t.id)}
                  aria-label="Mark undone"
                >
                  <Check size={10} strokeWidth={3} />
                </button>
                <span className="task-title">{t.title}</span>
                <button
                  type="button"
                  className="task-delete"
                  onClick={() => taskList.remove(t.id)}
                  aria-label="Delete task"
                >
                  <X size={12} />
                </button>
              </div>
            ))}
          </div>
        )}

        {tasks.length === 0 && (
          <div className="tasks-empty">
            <p>No tasks yet.</p>
            <span>Type above to keep a thought for later.</span>
          </div>
        )}
      </div>
    </div>
  );
};
