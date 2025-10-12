import { useFormContext } from "~/hooks/form-context";
import { Button } from "../ui/button";

export default function SubscribeButton({ label }: { label: string }) {
  const form = useFormContext();
  return (
    <form.Subscribe
      selector={(state) => [state.isSubmitting, state.isFormValid]}
    >
      {([isSubmitting, isFormValid]) => (
        <Button disabled={isSubmitting || !isFormValid}>{label}</Button>
      )}
    </form.Subscribe>
  );
}
