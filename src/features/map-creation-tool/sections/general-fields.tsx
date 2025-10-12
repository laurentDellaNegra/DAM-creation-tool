import { HStack } from "styled-system/jsx";
import {
  BadgeInfoIcon,
  CalendarIcon,
  CheckIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsUpDownIcon,
} from "lucide-react";
import { createListCollection, parseDate } from "@ark-ui/react";
import { css } from "styled-system/css";
import { withForm } from "~/hooks/form";
import { damFormOpts } from "../shared-form";
import { Fieldset } from "~/components/ui/fieldset";
import { DatePicker } from "~/components/ui/date-picker";
import { Input } from "~/components/ui/input";
import { IconButton } from "~/components/ui/icon-button";
import { Switch } from "~/components/ui/switch";
import { Select } from "~/components/ui/select";
import { Button } from "~/components/ui/button";

const collection = createListCollection({
  items: [
    { label: "LSZRH", value: "lszrh" },
    { label: "TRA", value: "tra" },
    { label: "Glider", value: "glider" },
    { label: "Para", value: "para", disabled: true },
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

          <form.AppField
            name="general.rangeDate"
            children={(field) => (
              <DatePicker.Root
                positioning={{ sameWidth: true }}
                startOfWeek={1}
                selectionMode="range"
                locale="en-CH"
                value={[
                  ...(field.state.value.startDate
                    ? [parseDate(field.state.value.startDate)]
                    : []),
                  ...(field.state.value.endDate
                    ? [parseDate(field.state.value.endDate)]
                    : []),
                ]}
                onValueChange={(e) =>
                  field.handleChange({
                    startDate: e.value?.[0]?.toString() ?? "",
                    endDate: e.value?.[1]?.toString() ?? "",
                  })
                }
              >
                <DatePicker.Label>Date Picker</DatePicker.Label>
                <DatePicker.Control gap={4}>
                  <DatePicker.Input index={0} asChild>
                    <Input />
                  </DatePicker.Input>
                  <DatePicker.Input index={1} asChild>
                    <Input />
                  </DatePicker.Input>
                  <DatePicker.Trigger asChild>
                    <IconButton variant="outline" aria-label="Open date picker">
                      <CalendarIcon />
                    </IconButton>
                  </DatePicker.Trigger>
                </DatePicker.Control>
                <DatePicker.Positioner>
                  <DatePicker.Content>
                    <DatePicker.View view="day">
                      <DatePicker.Context>
                        {(api) => (
                          <>
                            <DatePicker.ViewControl>
                              <DatePicker.PrevTrigger asChild>
                                <IconButton variant="ghost" size="sm">
                                  <ChevronLeftIcon />
                                </IconButton>
                              </DatePicker.PrevTrigger>
                              <DatePicker.ViewTrigger asChild>
                                <Button variant="ghost" size="sm">
                                  <DatePicker.RangeText />
                                </Button>
                              </DatePicker.ViewTrigger>
                              <DatePicker.NextTrigger asChild>
                                <IconButton variant="ghost" size="sm">
                                  <ChevronRightIcon />
                                </IconButton>
                              </DatePicker.NextTrigger>
                            </DatePicker.ViewControl>
                            <DatePicker.Table>
                              <DatePicker.TableHead>
                                <DatePicker.TableRow>
                                  {api.weekDays.map((weekDay, id) => (
                                    <DatePicker.TableHeader key={id}>
                                      {weekDay.narrow}
                                    </DatePicker.TableHeader>
                                  ))}
                                </DatePicker.TableRow>
                              </DatePicker.TableHead>
                              <DatePicker.TableBody>
                                {api.weeks.map((week, id) => (
                                  <DatePicker.TableRow key={id}>
                                    {week.map((day, id) => (
                                      <DatePicker.TableCell
                                        key={id}
                                        value={day}
                                      >
                                        <DatePicker.TableCellTrigger asChild>
                                          <IconButton variant="ghost">
                                            {day.day}
                                          </IconButton>
                                        </DatePicker.TableCellTrigger>
                                      </DatePicker.TableCell>
                                    ))}
                                  </DatePicker.TableRow>
                                ))}
                              </DatePicker.TableBody>
                            </DatePicker.Table>
                          </>
                        )}
                      </DatePicker.Context>
                    </DatePicker.View>
                    <DatePicker.View view="month">
                      <DatePicker.Context>
                        {(api) => (
                          <>
                            <DatePicker.ViewControl>
                              <DatePicker.PrevTrigger asChild>
                                <IconButton variant="ghost" size="sm">
                                  <ChevronLeftIcon />
                                </IconButton>
                              </DatePicker.PrevTrigger>
                              <DatePicker.ViewTrigger asChild>
                                <Button variant="ghost" size="sm">
                                  <DatePicker.RangeText />
                                </Button>
                              </DatePicker.ViewTrigger>
                              <DatePicker.NextTrigger asChild>
                                <IconButton variant="ghost" size="sm">
                                  <ChevronRightIcon />
                                </IconButton>
                              </DatePicker.NextTrigger>
                            </DatePicker.ViewControl>
                            <DatePicker.Table>
                              <DatePicker.TableBody>
                                {api
                                  .getMonthsGrid({
                                    columns: 4,
                                    format: "short",
                                  })
                                  .map((months, id) => (
                                    <DatePicker.TableRow key={id}>
                                      {months.map((month, id) => (
                                        <DatePicker.TableCell
                                          key={id}
                                          value={month.value}
                                        >
                                          <DatePicker.TableCellTrigger asChild>
                                            <Button variant="ghost">
                                              {month.label}
                                            </Button>
                                          </DatePicker.TableCellTrigger>
                                        </DatePicker.TableCell>
                                      ))}
                                    </DatePicker.TableRow>
                                  ))}
                              </DatePicker.TableBody>
                            </DatePicker.Table>
                          </>
                        )}
                      </DatePicker.Context>
                    </DatePicker.View>
                    <DatePicker.View view="year">
                      <DatePicker.Context>
                        {(api) => (
                          <>
                            <DatePicker.ViewControl>
                              <DatePicker.PrevTrigger asChild>
                                <IconButton variant="ghost" size="sm">
                                  <ChevronLeftIcon />
                                </IconButton>
                              </DatePicker.PrevTrigger>
                              <DatePicker.ViewTrigger asChild>
                                <Button variant="ghost" size="sm">
                                  <DatePicker.RangeText />
                                </Button>
                              </DatePicker.ViewTrigger>
                              <DatePicker.NextTrigger asChild>
                                <IconButton variant="ghost" size="sm">
                                  <ChevronRightIcon />
                                </IconButton>
                              </DatePicker.NextTrigger>
                            </DatePicker.ViewControl>
                            <DatePicker.Table>
                              <DatePicker.TableBody>
                                {api
                                  .getYearsGrid({ columns: 4 })
                                  .map((years, id) => (
                                    <DatePicker.TableRow key={id}>
                                      {years.map((year, id) => (
                                        <DatePicker.TableCell
                                          key={id}
                                          value={year.value}
                                        >
                                          <DatePicker.TableCellTrigger asChild>
                                            <Button variant="ghost">
                                              {year.label}
                                            </Button>
                                          </DatePicker.TableCellTrigger>
                                        </DatePicker.TableCell>
                                      ))}
                                    </DatePicker.TableRow>
                                  ))}
                              </DatePicker.TableBody>
                            </DatePicker.Table>
                          </>
                        )}
                      </DatePicker.Context>
                    </DatePicker.View>
                  </DatePicker.Content>
                </DatePicker.Positioner>
              </DatePicker.Root>
            )}
          />

          {/* DL */}
          <form.AppField
            name="general.dl"
            children={({ handleChange, state }) => (
              <Switch
                onCheckedChange={({ checked }) =>
                  handleChange(checked === true)
                }
                checked={state.value}
              >
                DL
              </Switch>
            )}
          />

          {/* Map combobox */}
          <form.AppField
            name="general.map"
            children={({ handleChange, state }) => (
              <Select.Root
                positioning={{ sameWidth: true }}
                width="2xs"
                collection={collection}
                onValueChange={(e) => handleChange(e.value[0])}
                value={state.value ? [state.value] : []}
              >
                <Select.Label>Map</Select.Label>
                <Select.Control>
                  <Select.Trigger>
                    <Select.ValueText placeholder="Select a map" />
                    <ChevronsUpDownIcon />
                  </Select.Trigger>
                </Select.Control>
                <Select.Positioner>
                  <Select.Content>
                    <Select.ItemGroup>
                      <Select.ItemGroupLabel>Static map</Select.ItemGroupLabel>
                      {collection.items.map((item) => (
                        <Select.Item key={item.value} item={item}>
                          <Select.ItemText>{item.label}</Select.ItemText>
                          <Select.ItemIndicator>
                            <CheckIcon />
                          </Select.ItemIndicator>
                        </Select.Item>
                      ))}
                    </Select.ItemGroup>
                  </Select.Content>
                </Select.Positioner>
              </Select.Root>
            )}
          />
        </Fieldset.Control>
      </Fieldset.Root>
    );
  },
});
