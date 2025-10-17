import { Grid, HStack, VStack } from "styled-system/jsx";
import { MapIcon } from "lucide-react";
import { css } from "styled-system/css";
import { Fieldset } from "../../../components/ui/fieldset";
import { withForm } from "~/hooks/form";
import { damFormOpts } from "../shared-form";

export const AltitudeCorrectionFields = withForm({
  ...damFormOpts,
  render: ({ form }) => {
    return (
      <Fieldset.Root>
        <Fieldset.Legend>
          <HStack>
            <MapIcon className={css({ w: "4", h: "4" })} />
            Altitude Corrections
          </HStack>
        </Fieldset.Legend>
        <Fieldset.HelperText>
          Select altitude correction methods and buffer settings.
        </Fieldset.HelperText>
        <Fieldset.Control gap={10}>
          <HStack gap="4" alignItems="flex-start">
            <HStack gap="4">
              {/* QNH-corr */}
              <form.AppField
                name="altitudeCorrection.qnhCorr"
                children={(field) => <field.CheckboxField label="QNH corr." />}
              />

              {/* FL Corr */}
              <form.AppField
                name="altitudeCorrection.flCorr"
                children={(field) => <field.CheckboxField label="FL corr." />}
              />
            </HStack>

            <VStack gap="4">
              {/* UL corr */}
              <form.AppField
                name="altitudeCorrection.ulHalfBuffer"
                children={(field) => (
                  <field.CheckboxField label="UL 1/2 buffer" />
                )}
              />

              {/* UL no buffer */}
              <form.AppField
                name="altitudeCorrection.ulNoBuffer"
                children={(field) => (
                  <field.CheckboxField label="UL No buffer" />
                )}
              />
            </VStack>

            <VStack gap="4">
              {/* LL corr */}
              <form.AppField
                name="altitudeCorrection.llHalfBuffer"
                children={(field) => (
                  <field.CheckboxField label="LL 1/2 buffer" />
                )}
              />

              {/* LL no buffer */}
              <form.AppField
                name="altitudeCorrection.llNoBuffer"
                children={(field) => (
                  <field.CheckboxField label="LL No buffer" />
                )}
              />
            </VStack>
          </HStack>
        </Fieldset.Control>
      </Fieldset.Root>
    );
  },
});
