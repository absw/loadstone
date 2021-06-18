module Main exposing (main)

import Model exposing (..)
import View exposing (..)
import Update exposing (..)
import Upload

import Browser exposing (document)

init : Flags -> (Model, Cmd Message)
init _ = (Model.default, Update.make_metrics_request)

subscriptions : Model -> Sub Message
subscriptions _ = Upload.notify UploadNotify

main : Program Flags Model Message
main =
    document {
        init = init,
        view = View.view,
        update = Update.update,
        subscriptions = subscriptions
    }
