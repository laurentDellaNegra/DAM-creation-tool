import { HStack } from "styled-system/jsx";
import { BadgeInfoIcon } from "lucide-react";
import { createListCollection } from "@ark-ui/react";
import { css } from "styled-system/css";
import { withForm } from "~/hooks/form";
import { damFormOpts } from "../shared-form";
import { Fieldset } from "~/components/ui/fieldset";

const collection = createListCollection({
  items: [
    { label: "LSZRH", value: "lszrh" },
    { label: "TRA", value: "tra" },
    { label: "Glider", value: "glider" },
    { label: "Para", value: "para" },
  ],
});

export const GeneralFields = withForm({
  ...damFormOpts,
  render: ({ form }) => {
    return (
      <Fieldset.Root>
        <Fieldset.Legend>
          <HStack>
            <BadgeInfoIcon className={css({ w: "4", h: "4" })} />
            Basic Information
          </HStack>
        </Fieldset.Legend>
        <Fieldset.HelperText>
          Select a DAM map name, a date and whether it is a static map or
          dynamic map.
        </Fieldset.HelperText>
        <Fieldset.Control>
          {/* Map Name */}
          <form.AppField
            name="general.mapName"
            children={(field) => (
              <field.TextField label="Map Name" placeholder="Enter map name" />
            )}
          />

          {/* Date selection */}
          <form.AppField
            name="general.rangeDate"
            children={(field) => (
              <field.RangePickerField label="Validity" locale="en-CH" />
            )}
          />

          {/* DL */}
          <form.AppField
            name="general.dl"
            children={(field) => <field.SwitchField>DL</field.SwitchField>}
          />

          {/* Map combobox */}
          <form.AppField
            name="general.map"
            children={(field) => (
              <field.SelectField collection={collection} label="Map" />
            )}
          />
        </Fieldset.Control>
      </Fieldset.Root>
    );
  },
});
