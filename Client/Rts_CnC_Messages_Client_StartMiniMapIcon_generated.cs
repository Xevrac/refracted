using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_StartMiniMapIcon
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.StartMiniMapIcon); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.StartMiniMapIcon)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Style
            s.Write(value.Style);
            //  Serialize MiniMapIconId
            s.Write(value.MiniMapIconId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.StartMiniMapIcon)) as Rts.CnC.Messages.Client.StartMiniMapIcon;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Style
            s.Read(out value.Style);
            //  Deserialize MiniMapIconId
            s.Read(out value.MiniMapIconId);

            return value;
        }
        
    }
}
