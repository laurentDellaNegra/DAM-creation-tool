import { HStack } from "styled-system/jsx";
import { Field } from "../ui/field";
import { Fieldset } from "../ui/fieldset";
import { DatePicker } from "../ui/date-picker";
import { Input } from "../ui/input";
import { IconButton } from "../ui/icon-button";
import {
  BadgeInfoIcon,
  CalendarIcon,
  CheckIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsUpDownIcon,
} from "lucide-react";
import { Button } from "../ui/button";
import { Switch } from "../ui/switch";
import { createListCollection } from "@ark-ui/react";
import { Select } from "../ui/select";
import { css } from "styled-system/css";

export default function BasicSection() {
  const collection = createListCollection({
    items: [
      { label: "LSZRH", value: "lszrh" },
      { label: "TRA", value: "tra" },
      { label: "Glider", value: "glider" },
      { label: "Para", value: "para", disabled: true },
    ],
  });
  return (
    <Fieldset.Root>
      <Fieldset.Legend>
        <HStack>
          <BadgeInfoIcon className={css({ w: "4", h: "4" })} />
          Basic Information
        </HStack>
      </Fieldset.Legend>
      <Fieldset.HelperText>
        Select a DAM map name, a date and whether it is a static map or dynamic
        map.
      </Fieldset.HelperText>
      <Fieldset.Control>
        <Field.Root>
          <Field.Label>Map Name</Field.Label>
          <Field.Input placeholder="Enter map name" />
          <Field.HelperText>Enter a unique name for your map</Field.HelperText>
        </Field.Root>

        <HStack>
          <DatePicker.Root
            positioning={{ sameWidth: true }}
            startOfWeek={1}
            selectionMode="range"
            locale="en-CH"
          >
            <DatePicker.Label>Date Picker</DatePicker.Label>
            <DatePicker.Control gap={10}>
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
                                  <DatePicker.TableCell key={id} value={day}>
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
        </HStack>

        <Switch>DL</Switch>

        <Select.Root
          positioning={{ sameWidth: true }}
          width="2xs"
          collection={collection}
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
      </Fieldset.Control>
    </Fieldset.Root>
  );
}
