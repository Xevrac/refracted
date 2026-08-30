using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_StopMiniMapIcon
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.StopMiniMapIcon); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.StopMiniMapIcon)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize MiniMapIconId
            s.Write(value.MiniMapIconId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.StopMiniMapIcon)) as Rts.CnC.Messages.Client.StopMiniMapIcon;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize MiniMapIconId
            s.Read(out value.MiniMapIconId);

            return value;
        }
        
    }
}
