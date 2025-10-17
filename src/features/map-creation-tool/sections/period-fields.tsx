import { HStack } from "styled-system/jsx";
import { Button } from "~/components/ui/button";
import { ClockIcon, Plus, Trash } from "lucide-react";
import { css } from "styled-system/css";
import { Fieldset } from "../../../components/ui/fieldset";
import { withForm } from "~/hooks/form";
import { damFormOpts } from "../shared-form";
import { Fragment } from "react/jsx-runtime";

export const PeriodFields = withForm({
  ...damFormOpts,
  render: ({ form }) => {
    return (
      <Fieldset.Root>
        <Fieldset.Legend>
          <HStack>
            <ClockIcon className={css({ w: "4", h: "4" })} />
            Today/Repetitive Periods
          </HStack>
        </Fieldset.Legend>
        <Fieldset.HelperText>
          Configure start and end periods with altitude limits in feet (ft) or
          flight level (FL).
        </Fieldset.HelperText>
        <Fieldset.Control gap={10}>
          <form.Field name="periods" mode="array">
            {(field) => {
              return (
                <HStack>
                  {field.state.value.map((_, i) => {
                    return (
                      <Fragment key={i}>
                        {/* start time */}
                        <form.AppField
                          name={`periods[${i}].startTime`}
                          children={(field) => (
                            <field.TextField
                              label="Start time"
                              placeholder="09:00"
                              maxLength={5}
                              width={20}
                            />
                          )}
                        />

                        {/* end time */}
                        <form.AppField
                          name={`periods[${i}].startTime`}
                          children={(field) => (
                            <field.TextField
                              label="Start time"
                              placeholder="10:00"
                              maxLength={5}
                              width={20}
                            />
                          )}
                        />

                        <HStack alignItems="flex-end">
                          {/* Lower limit */}
                          <form.AppField
                            name={`periods[${i}].lowerLimit`}
                            children={(field) => (
                              <field.TextField
                                label="Lower Limit"
                                maxLength={5}
                                width={20}
                                type="number"
                              />
                            )}
                          />

                          {/* Lower limit is feet */}
                          <form.AppField
                            name={`periods[${i}].lowerLimitIsFeet`}
                            children={(field) => (
                              <field.CheckboxField label="Feet" />
                            )}
                          />
                        </HStack>

                        <HStack alignItems="flex-end">
                          {/* Upper limit */}
                          <form.AppField
                            name={`periods[${i}].upperLimit`}
                            children={(field) => (
                              <field.TextField
                                label="Upper Limit"
                                maxLength={5}
                                width={20}
                                type="number"
                              />
                            )}
                          />

                          {/* Upper limit is feet */}
                          <form.AppField
                            name={`periods[${i}].upperLimitIsFeet`}
                            children={(field) => (
                              <field.CheckboxField label="Feet" />
                            )}
                          />
                        </HStack>
                      </Fragment>
                    );
                  })}
                </HStack>
              );
            }}
          </form.Field>

          <HStack>
            <Button variant="outline" ml="auto">
              <Plus />
              Add Period
            </Button>
            <Button variant="outline">
              <Trash />
              Remove Period
            </Button>
          </HStack>
        </Fieldset.Control>
      </Fieldset.Root>
    );
  },
});
