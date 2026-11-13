import { useEffect, useRef } from "react";
import { Icon } from "./Icon";
import { checkIcon } from "./icons";
import styles from "./Checkbox.module.scss";

export function Checkbox({ checked, indeterminate, disabled, label, ariaLabel, onChange }: {
  checked: boolean;
  /** 半选状态(例如分组内只有部分项被选中);为 true 时显示横线而不是勾。 */
  indeterminate?: boolean;
  disabled?: boolean;
  /** 复选框旁的可见文案;省略时只显示复选框本身。 */
  label?: string;
  /** 省略可见文案时的无障碍名称。 */
  ariaLabel?: string;
  onChange: (checked: boolean) => void;
}) {
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (input.current) input.current.indeterminate = Boolean(indeterminate) && !checked;
  }, [checked, indeterminate]);
  return <label className={styles.root}>
    <input ref={input} type="checkbox" checked={checked} disabled={disabled} aria-label={ariaLabel} onChange={(event) => onChange(event.target.checked)} />
    <span className={styles.control} aria-hidden="true">
      {checked ? <Icon icon={checkIcon} size="0.875em" /> : indeterminate && <span className={styles.dash} />}
    </span>
    {label && <span>{label}</span>}
  </label>;
}
