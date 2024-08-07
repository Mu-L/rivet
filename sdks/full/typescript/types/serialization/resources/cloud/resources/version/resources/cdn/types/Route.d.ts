/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { cloud } from "../../../../../../index";
export declare const Route: core.serialization.ObjectSchema<serializers.cloud.version.cdn.Route.Raw, Rivet.cloud.version.cdn.Route>;
export declare namespace Route {
    interface Raw {
        glob: string;
        priority: number;
        middlewares: cloud.version.cdn.Middleware.Raw[];
    }
}
