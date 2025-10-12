import { useStore } from "@tanstack/react-form";
import { useFieldContext } from "~/hooks/form-context";
import { Field } from "../ui/field";

export type TextFieldProps = {
  label: string;
  placeholder?: string;
  helperText?: string;
};

export default function TextField({
  label,
  placeholder,
  helperText,
}: TextFieldProps) {
  const field = useFieldContext<string>();

  const errors = useStore(field.store, (state) => state.meta.errors);

  return (
    <Field.Root invalid={errors.length > 0}>
      <Field.Label>{label}</Field.Label>
      <Field.Input
        placeholder={placeholder}
        value={field.state.value}
        onChange={(e) => field.handleChange(e.target.value)}
        onBlur={field.handleBlur}
      />
      <Field.HelperText>{helperText}</Field.HelperText>
      {errors.map((error) => (
        <Field.ErrorText key={error}>{error.message}</Field.ErrorText>
      ))}
    </Field.Root>
  );
}
