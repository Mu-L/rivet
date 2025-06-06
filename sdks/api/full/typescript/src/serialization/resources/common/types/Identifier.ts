/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";

export const Identifier: core.serialization.Schema<serializers.Identifier.Raw, Rivet.Identifier> =
    core.serialization.string();

export declare namespace Identifier {
    export type Raw = string;
}
