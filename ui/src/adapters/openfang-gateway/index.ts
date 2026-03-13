import type { UIAdapterModule } from "../types";
import { parseOpenFangGatewayStdoutLine } from "./parse-stdout";
import { buildOpenFangGatewayConfig } from "./build-config";
import { OpenFangGatewayConfigFields } from "./config-fields";

export const openFangGatewayUIAdapter: UIAdapterModule = {
  type: "openfang_gateway",
  label: "OpenFang Gateway",
  parseStdoutLine: parseOpenFangGatewayStdoutLine,
  ConfigFields: OpenFangGatewayConfigFields,
  buildAdapterConfig: buildOpenFangGatewayConfig,
};
