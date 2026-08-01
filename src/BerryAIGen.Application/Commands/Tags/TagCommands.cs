using MediatR;

namespace BerryAIGen.Application.Commands.Tags;

public record CreateTagCommand : IRequest<Guid>
{
    public string Name { get; init; } = string.Empty;
    public string? Color { get; init; }
    public string? Category { get; init; }
}

public record UpdateTagCommand : IRequest<Unit>
{
    public Guid Id { get; init; }
    public string Name { get; init; } = string.Empty;
    public string? Color { get; init; }
    public string? Category { get; init; }
}

public record DeleteTagCommand : IRequest<Unit>
{
    public Guid Id { get; init; }
}