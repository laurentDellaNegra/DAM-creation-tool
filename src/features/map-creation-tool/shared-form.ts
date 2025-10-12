import { formOptions } from "@tanstack/react-form";
import { damSchema } from "./dam-schemas";

export const damFormOpts = formOptions({
  defaultValues: {
    general: {
      mapName: "",
      rangeDate: {
        startDate: "",
        endDate: "",
      },
      map: "",
      dl: false,
    },
    periods: [
      {
        startTime: "",
        endTime: "",
        lowerLimit: 0,
        upperLimit: 99999,
        lowerLimitIsFeet: false,
        upperLimitIsFeet: false,
      },
    ],
    altitudeCorrection: {
      qnhCorr: false,
      flCorr: false,
      ulHalfBuffer: false,
      llHalfBuffer: false,
      ulNoBuffer: false,
      llNoBuffer: false,
    },
    additionalInformation: {
      text: "",
      dabsInfo: "",
    },
  },
  validators: {
    onChange: damSchema,
  },
});
