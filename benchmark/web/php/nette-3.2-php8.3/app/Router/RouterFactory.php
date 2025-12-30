<?php

declare(strict_types=1);

namespace App\Router;

use Nette\Application\Routers\RouteList;

final class RouterFactory
{
    public static function createRouter(): RouteList
    {
        $router = new RouteList;

        $router->addRoute('/api/v1/periodic-table/element', [
            'presenter' => 'Api',
            'action' => 'element',
        ]);

        $router->addRoute('/api/v1/periodic-table/shells', [
            'presenter' => 'Api',
            'action' => 'shells',
        ]);

        return $router;
    }
}
