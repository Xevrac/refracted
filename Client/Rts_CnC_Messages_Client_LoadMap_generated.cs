using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_LoadMap
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.LoadMap); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.LoadMap)obj;
            //  Serialize PlayerHandle
            s.Write(value.PlayerHandle);
            //  Serialize MapId
            s.Write(value.MapId);
            //  Serialize UILayout
            s.Write(value.UILayout);
            //  Serialize UILoadingScreen
            s.Write(value.UILoadingScreen);
            //  Serialize CameraPos
            s.Write(value.CameraPos);
            //  Serialize CameraRotation
            s.Write(value.CameraRotation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.LoadMap)) as Rts.CnC.Messages.Client.LoadMap;
            //  Deserialize PlayerHandle
            s.Read(out value.PlayerHandle);
            //  Deserialize MapId
            s.Read(out value.MapId);
            //  Deserialize UILayout
            s.Read(out value.UILayout);
            //  Deserialize UILoadingScreen
            s.Read(out value.UILoadingScreen);
            //  Deserialize CameraPos
            s.Read(out value.CameraPos);
            //  Deserialize CameraRotation
            s.Read(out value.CameraRotation);

            return value;
        }
        
    }
}
