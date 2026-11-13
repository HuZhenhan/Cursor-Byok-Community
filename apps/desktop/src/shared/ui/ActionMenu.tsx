import { autoUpdate, computePosition, flip, offset, shift } from "@floating-ui/dom";
import { useEffect, useId, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import { createPortal } from "react-dom";
import { Button } from "./Button";
import { Icon } from "./Icon";
import { chevronDownIcon, moreIcon } from "./icons";
import styles from "./ActionMenu.module.scss";
import controls from "./Controls.module.scss";

export type ActionMenuItem =
  | {
      id: string;
      label: string;
      type?: "action";
      disabled?: boolean;
      onSelect: () => void;
    }
  | {
      id: string;
      label: string;
      type: "text";
    };

/** 触发器 + 动作列表的下拉菜单,用于容纳卡片上的次要操作。 */
export function ActionMenu({ label, items, disabled, iconOnly }: {
  label: string;
  items: ActionMenuItem[];
  disabled?: boolean;
  /** 仅显示三点图标,适合放在列表行右侧的窄操作列。 */
  iconOnly?: boolean;
}) {
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const menuId = useId();
  const [open, setOpen] = useState(false);
  const [position, setPosition] = useState({ left: 0, top: 0 });

  useLayoutEffect(() => {
    if (!open || !trigger.current || !menu.current) return;
    return autoUpdate(trigger.current, menu.current, () =>
      void computePosition(trigger.current!, menu.current!, {
        placement: "bottom-end",
        middleware: [offset(5), flip({ padding: 10 }), shift({ padding: 10 })],
      }).then(({ x, y }) => setPosition({ left: x, top: y })));
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const outside = (event: PointerEvent) => {
      if (!trigger.current?.contains(event.target as Node) && !menu.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, [open]);

  const close = () => {
    setOpen(false);
    trigger.current?.focus();
  };

  const toggle = () => setOpen((current) => !current);
  const onTriggerKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "Escape") close();
  };

  return <>
    {iconOnly
      ? <button
        ref={trigger}
        type="button"
        className={controls.iconButton}
        disabled={disabled}
        aria-label={label}
        title={label}
        aria-haspopup="menu"
        aria-controls={open ? menuId : undefined}
        aria-expanded={open}
        onClick={toggle}
        onKeyDown={onTriggerKeyDown}
      >
        <Icon icon={moreIcon} size="1.1em" />
      </button>
      : <Button
        ref={trigger}
        size="small"
        disabled={disabled}
        aria-haspopup="menu"
        aria-controls={open ? menuId : undefined}
        aria-expanded={open}
        onClick={toggle}
        onKeyDown={onTriggerKeyDown}
      >
        {label}
        <Icon icon={chevronDownIcon} size="1em" className={open ? styles.openIcon : undefined} />
      </Button>}
    {open && createPortal(
      <div
        id={menuId}
        ref={menu}
        className={styles.menu}
        role="menu"
        style={{ left: position.left, top: position.top }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            close();
          }
        }}
      >
        {items.map((item) => item.type === "text" ? (
          <span key={item.id} className={styles.textItem}>
            {item.label}
          </span>
        ) : (
          <button
            key={item.id}
            type="button"
            role="menuitem"
            disabled={item.disabled}
            onClick={() => {
              setOpen(false);
              item.onSelect();
            }}
          >
            {item.label}
          </button>
        ))}
      </div>,
      document.body,
    )}
  </>;
}
