import {_AST_MAP as protoclient_protoapp_api} from "./protoapp/api";
import {_AST_MAP as protoclient_protoapp_config} from "./protoapp/config";
import {_AST_MAP as protoclient_protoapp_db} from "./protoapp/db";
import {_AST_MAP as protoclient_protoapp_uiconfig} from "./protoapp/uiconfig";
import {_AST_MAP as common} from "@adl-lang/common/";
import {_AST_MAP as common_config_aws} from "@adl-lang/common/config/aws";
import {_AST_MAP as common_config_db} from "@adl-lang/common/config/db";
import {_AST_MAP as common_config_frontend} from "@adl-lang/common/config/frontend";
import {_AST_MAP as common_config_log} from "@adl-lang/common/config/log";
import {_AST_MAP as common_db} from "@adl-lang/common/db";
import {_AST_MAP as common_flyway_internals} from "@adl-lang/common/flyway/internals";
import {_AST_MAP as common_http} from "@adl-lang/common/http";
import {_AST_MAP as common_strings} from "@adl-lang/common/strings";
import {_AST_MAP as common_tabular} from "@adl-lang/common/tabular";
import {_AST_MAP as common_ui} from "@adl-lang/common/ui";
import {ScopedDecl, declResolver} from "@adl-lang/runtime/adl";
import {_AST_MAP as sys_annotations} from "@adl-lang/sys/annotations";
import {_AST_MAP as sys_types} from "@adl-lang/sys/types";

/* @generated from adl */

export const ADL_local: { [key: string]: ScopedDecl } = {
  ...common,
  ...common_config_aws,
  ...common_config_db,
  ...common_config_frontend,
  ...common_config_log,
  ...common_db,
  ...common_flyway_internals,
  ...common_http,
  ...common_strings,
  ...common_tabular,
  ...common_ui,
  ...protoclient_protoapp_api,
  ...protoclient_protoapp_config,
  ...protoclient_protoapp_db,
  ...protoclient_protoapp_uiconfig,
  ...sys_annotations,
  ...sys_types,
};

export const ADL: { [key: string]: ScopedDecl } = {
  ...ADL_local,
};

export const RESOLVER = declResolver(ADL);
