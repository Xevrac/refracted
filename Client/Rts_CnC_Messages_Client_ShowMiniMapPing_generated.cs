using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ShowMiniMapPing
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ShowMiniMapPing); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ShowMiniMapPing)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Style
            s.Write(value.Style);
            //  Serialize DurationMs
            s.Write(value.DurationMs);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ShowMiniMapPing)) as Rts.CnC.Messages.Client.ShowMiniMapPing;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Style
            s.Read(out value.Style);
            //  Deserialize DurationMs
            s.Read(out value.DurationMs);

            return value;
        }
        
    }
}
