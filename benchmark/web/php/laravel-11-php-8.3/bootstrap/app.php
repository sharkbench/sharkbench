<?php

use Illuminate\Foundation\Application;
use Illuminate\Foundation\Configuration\Exceptions;
use Illuminate\Foundation\Configuration\Middleware;
use Illuminate\Http\Request;
use Illuminate\Session\Middleware\StartSession;

class CustomStartSession extends StartSession
{
    protected function handleStatefulRequest(Request $request, $session, Closure $next)
    {
        $request->setLaravelSession(
            $this->startSession($request, $session)
        );

        $this->collectGarbage($session);
        $response = $next($request);
        $this->storeCurrentUrl($request, $session);

        $this->saveSession($request);
        return $response;
    }
}

return Application::configure(basePath: dirname(__DIR__))
    ->withRouting(
        web: __DIR__.'/../routes/web.php',
        commands: __DIR__.'/../routes/console.php',
        health: '/up',
    )
    ->withMiddleware(function (Middleware $middleware) {
        // disable session
        $middleware->web(replace: [StartSession::class => CustomStartSession::class]);

        // TODO: remove CSRF
    })
    ->withExceptions(function (Exceptions $exceptions) {
        //
    })->create();
