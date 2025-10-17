import { useStore } from "@tanstack/react-form";
import { useFieldContext } from "~/hooks/form-context";
import { Checkbox, type CheckboxProps } from "../ui/checkbox";
import { css } from "styled-system/css";

export interface CheckboxFieldProps extends CheckboxProps {
  label: string;
}

export default function CheckboxField({ label, ...rest }: CheckboxFieldProps) {
  const field = useFieldContext<boolean>();

  const errors = useStore(field.store, (state) => state.meta.errors);

  console.log(errors);

  return (
    <Checkbox
      {...rest}
      onCheckedChange={({ checked }) => field.handleChange(checked === true)}
      checked={field.state.value}
    >
      <span className={css({ fontSize: "sm" })}>{label}</span>
    </Checkbox>
  );
}
