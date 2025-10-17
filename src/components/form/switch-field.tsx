import { useFieldContext } from "~/hooks/form-context";
import { Switch, type SwitchProps } from "../ui/switch";

export default function SwitchField(props: SwitchProps) {
  const field = useFieldContext<boolean>();

  return (
    <Switch
      {...props}
      onCheckedChange={({ checked }) => field.handleChange(checked === true)}
      checked={field.state.value}
    />
  );
}
