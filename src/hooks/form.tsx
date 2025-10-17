import { createFormHook } from "@tanstack/react-form";
import { fieldContext, formContext } from "./form-context.ts";
import SubscribeButton from "~/components/form/subscribe-button.tsx";
import TextField from "~/components/form/text-field.tsx";
import RangePickerField from "~/components/form/range-picker-field.tsx";
import SwitchField from "~/components/form/switch-field.tsx";
import SelectField from "~/components/form/select-field.tsx";
import CheckboxField from "~/components/form/checkbox-field.tsx";

export const { useAppForm, withForm, withFieldGroup } = createFormHook({
  fieldComponents: {
    CheckboxField,
    SelectField,
    SwitchField,
    RangePickerField,
    TextField,
  },
  formComponents: {
    SubscribeButton,
  },
  fieldContext,
  formContext,
});
