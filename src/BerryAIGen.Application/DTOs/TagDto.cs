namespace BerryAIGen.Application.DTOs;

public record TagDto
{
    public Guid Id { get; init; }
    public string Name { get; init; } = string.Empty;
    public string? Color { get; init; }
    public string? Category { get; init; }
    public DateTime CreatedAt { get; init; }
}

public record TagSummaryDto
{
    public Guid Id { get; init; }
    public string Name { get; init; } = string.Empty;
    public string? Color { get; init; }
    public string? Category { get; init; }
}