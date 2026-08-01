namespace BerryAIGen.Application.DTOs;

public record FolderDto
{
    public Guid Id { get; init; }
    public string Path { get; init; } = string.Empty;
    public Guid? ParentFolderId { get; init; }
    public bool IsWatched { get; init; }
    public bool Recursive { get; init; }
    public int ImageCount { get; init; }
    public DateTime CreatedAt { get; init; }
}

public record FolderSummaryDto
{
    public Guid Id { get; init; }
    public string Path { get; init; } = string.Empty;
    public bool IsWatched { get; init; }
    public int ImageCount { get; init; }
}