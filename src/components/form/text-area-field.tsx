import { useStore } from "@tanstack/react-form";
import { useFieldContext } from "~/hooks/form-context";
import { Field } from "../ui/field";
import type { FieldTextareaProps } from "@ark-ui/react";
import { Textarea } from "../ui/textarea";

export type TextAreaFieldProps = {
  label: string;
  placeholder?: string;
  helperText?: string;
  rows?: FieldTextareaProps["rows"];
};

export default function TextAreaField({
  label,
  placeholder,
  helperText,
  rows,
}: TextAreaFieldProps) {
  const field = useFieldContext<string>();

  const errors = useStore(field.store, (state) => state.meta.errors);

  return (
    <Field.Root invalid={errors.length > 0}>
      <Field.Label>{label}</Field.Label>
      <Textarea
        placeholder={placeholder}
        value={field.state.value}
        onChange={(e) => field.handleChange(e.target.value)}
        onBlur={field.handleBlur}
        rows={rows}
      />
      <Field.HelperText>{helperText}</Field.HelperText>
      {errors.map((error) => (
        <Field.ErrorText key={error}>{error.message}</Field.ErrorText>
      ))}
    </Field.Root>
  );
}
