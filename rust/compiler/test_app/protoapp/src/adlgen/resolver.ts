import {_AST_MAP as common_config_aws} from "./common/config/aws";
import {_AST_MAP as common_config_log} from "./common/config/log";
import {_AST_MAP as common_db} from "./common/db";
import {_AST_MAP as common_http} from "./common/http";
import {_AST_MAP as common_strings} from "./common/strings";
import {_AST_MAP as common_tabular} from "./common/tabular";
import {_AST_MAP as common_types} from "./common/types";
import {_AST_MAP as common_ui} from "./common/ui";
import {_AST_MAP as protoclient_protoapp_api} from "./protoclient/protoapp/api";
import {_AST_MAP as protoclient_protoapp_db} from "./protoclient/protoapp/db";
import {ScopedDecl, declResolver} from "./runtime/adl";
import {_AST_MAP as sys_annotations} from "./sys/annotations";
import {_AST_MAP as sys_types} from "./sys/types";

/* @generated from adl */

export const ADL_local: { [key: string]: ScopedDecl } = {
  ...common_config_aws,
  ...common_config_log,
  ...common_db,
  ...common_http,
  ...common_strings,
  ...common_tabular,
  ...common_types,
  ...common_ui,
  ...protoclient_protoapp_api,
  ...protoclient_protoapp_db,
  ...sys_annotations,
  ...sys_types,
};

export const ADL: { [key: string]: ScopedDecl } = {
  ...ADL_local,
};

export const RESOLVER = declResolver(ADL);
